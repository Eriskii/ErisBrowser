# Object-literal `super` properties

Ordinary object-literal concise methods, getters and setters now support
`super.name` and `super[key]`, including their parameter defaults and lexical
arrows. They reuse the [class `super` property operations](class-super-properties.md):
reads, calls, assignments, compound/logical assignments, updates, constructor
property calls and loop assignment targets. `delete super.x` is accepted syntax
and throws ReferenceError when evaluated. Direct `super()` remains forbidden
in object methods, including their defaults and arrows.

Each method retains the actual object where it was defined as its home.
The [DefineMethod](https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#sec-method-definitions-runtime-semantics-definemethod)
and [MakeMethod](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-makemethod)
algorithms describe method creation and attachment of that home.
Copying, borrowing or binding the function changes neither that identity nor
its lookup origin. Each new `super` reference reads the home's current prototype,
so a later `Object.setPrototypeOf` is observable. The call receiver is separate:
base getters, setters and method calls receive the method's actual `this`.
An escaped arrow keeps both the defining home and its lexical receiver.

Object methods retain ordinary strict/sloppy behavior. Sloppy methods perform
normal receiver conversion, including boxing primitive receivers; strict
methods retain the supplied receiver. Failed writes follow that method's
strictness: strict writes throw TypeError, while sloppy writes can return their
assignment value without changing the property. A writable base data property
examines the receiver's own descriptor; it does not invoke a setter found on
the receiver's prototype. Base accessors use the real receiver. Existing Array,
TypedArray, mapped-arguments and represented DOM routes keep their established
behavior and limitations.

Computed definition names run in the surrounding context before the new
method receives its own home. Defaults and the method body use the new home;
nested ordinary functions reset the `super` context, while lexical arrows
inherit it. Definitions retain source order and getter/setter merging. The
special `__proto__: value` form changes the object's prototype for an object or
null value, including when it appears after a method. Computed `__proto__` keys
and methods/accessors with that name remain ordinary properties. Duplicate
special prototype definitions remain syntax errors.

The [SuperProperty evaluation algorithm](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-super-keyword)
captures the receiver before a computed key's effects and the home prototype
after those effects. The raw key
is converted during the property operation. Simple assignment therefore runs
its right-hand side before conversion; compound assignments and updates reuse
the key converted for their read. Logical assignments skip both the right-hand
side and the write when they short-circuit. Calls preserve the receiver through
parentheses; a comma expression loses that reference. Callee getters run before
arguments. These operations retain the existing class-reference ordering and
ordinary thrown-value identity.

Attaching the defining home charges **eight work units and no additional heap**.
Admission and eligibility checks precede the single home publication; attachment
runs after function creation and before naming or property publication. Existing
function storage already pays for its home slot, and object creation, metadata,
property definitions, lookup and receiver operations retain their charges. This
is a logical accounting statement, not a claim that creating a method is free.
Runtime representations and limits are unchanged: 100,000 author work units,
1,000,000 trusted initialization units and 8 MiB of cumulative heap accounting.
Natural Resource and Unsupported failures retain reached effects and suppress
later bodies, catch/finally work and queued jobs under the existing observer.
A private attachment refusal alone does not establish that terminal boundary.

Optional chaining remains unsupported. A bounded parser correction recognizes
its adjacent `?.` prefix at supported expression boundaries and returns
Unsupported instead of misreading it as a conditional expression. Existing
context errors and the forbidden unparenthesized optional constructor prefix
retain SyntaxError. This is refusal classification, not parsing or validation
of the unimplemented optional-chain tail. Async/generator methods, object
spread, tagged templates, fields/private elements/static blocks, Proxy objects,
multiple realms and the other documented gaps remain outside this slice.

Runner/importer/protocol code, profile features, authenticated corpora and
baseline files are outside this runtime change. This chapter does not claim
complete ECMAScript conformance, a security certification, a performance
comparison or new GPU correctness evidence.

Final validation passes **3,485 tests with Rust 1.88 default features** and
**3,849 with Rust 1.99 and `test262-host,vulkan-raster`**, with no failures or
ignored tests across 56 suites in each run. Both final strict Clippy checks and
rustfmt pass. The 53 added groups comprise 41 independent public groups in 82
sloppy/strict modes, six parser groups, four private groups covering 58 measured
home-attachment calls, and two browser witnesses. The browser checks exercise
direct Page and confined-worker behavior; native-plan assertions run on the CPU.
No hardware GPU check was added.

The evidence retains the parent public result of ten passing and 31 failing
groups and the parent browser failure. The first candidate focused run passed
47 groups and failed one: the frozen optional-chain source was classified as
SyntaxError. The bounded prefix correction described above keeps that original
oracle. The first full attempt then stopped after 3,138 library passes and one
failure in an older aggregate test. Its exact object-method `super` literal
moved from an Unsupported list to a successful parse assertion; every other
row and all production bytes were retained. These failures remain recorded.

Temporary toolchain and Cargo-cache restoration were needed before the final
lint checks; the preflight/dependency failures did not require source changes.
The source review also retains a checker correction for one optional trailing
comma removed by rustfmt. That correction changed no source or test oracle.

Fresh ordinary and host release binaries replay all eight published profiles:
**1,330 cases comprise 1,258 passed, 64 failed and eight unsupported**. Every
complete case record and all **724 control records** match the published
`db880b86` predecessor exactly, including raw observations and classifications.
There are no transitions, prior-pass regressions or control changes. The
comparison recheck is byte-identical to the first summary. Existing failed and
unsupported outcomes remain visible; six profile commands return their expected
nonzero status for those failures. No baseline was rewritten.

The [evidence manifest](evidence/object-super-properties/manifest.json) and
[retained sources and checks](evidence/object-super-properties/sources-and-checks.tar.gz)
bind the final 15 changed source paths, independent contracts, retained attempts,
release provenance and complete replay comparisons. The 15,297 selected inputs
include unchanged tooling and authenticated corpora; this inventory is not a
self-contained external build graph.
