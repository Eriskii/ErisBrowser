# JavaScript classes

The interpreter now supports a bounded class subset: named declarations, named
and anonymous expressions, base and derived constructors, public instance and
static methods, getters and setters, and literal or computed string/Symbol
keys. Class syntax uses flat compiled records and parser/runtime continuations.
The behavior follows the selected [class definition](https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#sec-runtime-semantics-classdefinitionevaluation)
and [constructor](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-ecmascript-function-objects-construct-argumentslist-newtarget)
algorithms; this is not complete class or ECMAScript conformance.

Class declarations have a mutable outer lexical binding. An explicit class name
also has a distinct immutable inner binding, which remains in its temporal dead
zone while heritage and computed member names are evaluated. Anonymous classes
receive their own strict lexical environment too. Named evaluation supplies
inferred constructor names before static members are defined, so a later static
`name` or `length` definition is preserved. Methods are strict and cannot be used
as constructors. Member definitions preserve source order and ordinary data or
accessor descriptor behavior, including paired getters/setters and computed
Symbol keys.

Calling a class without `new` throws before parameter defaults or its body.
Base construction reads the original `new.target` prototype and allocates the
receiver before parameter evaluation. Derived construction starts with
uninitialized `this`: returning an object without calling `super()` is valid;
returning a non-undefined primitive throws, and falling through without an
initialized receiver throws ReferenceError.

Direct `super()` captures the current internal superclass before evaluating its
arguments, then checks that the captured value is a constructor before generic
bound forwarding. Parent construction happens before the duplicate-`super()`
binding check, so a second parent call can perform effects or throw first.
Default derived constructors validate the live parent before forwarding the
incoming argument list directly without reading
an Array iterator. Explicit rest/spread constructors use the existing iterable
machinery. Bound constructors and alternate `new.target` retain their existing
rules. An escaped arrow keeps the constructor activation, original `new.target`
and uninitialized receiver even after an object was returned; a later `super()`
uses the then-current internal superclass.

Native Promise subclasses keep genuine Promise state and existing capability,
species, reaction and microtask behavior. This adds no new Promise host protocol
or upstream feature admission. Existing class-tagged upstream rows remain
excluded by unchanged profile policies; results in older chapters belong to
their recorded checkpoints.

Fields, private elements, static blocks, `super` property access, generator and
async methods remain explicitly unsupported. Modules, decorators, async
functions, multiple realms and the other documented language gaps remain
outside this slice.

The selected accounting preserves existing helper tariffs and adds class-only
admission for names, environments, factories, member publication and constructor
state. UTF-16 name storage and arena/order-vector growth are prepaid. Earlier
author effects remain when later admission refuses. Public terminal failures
stop catch/finally work and leave unstarted jobs suspended under the existing
observer; a private helper alone does not create that observer boundary.

Four complete-shape probes observed an Environment increase from 96 to 112 bytes.
Its retained logical allowance rises from 176 to 192 bytes, adding 32 accounted
bytes for the two initial environments. The compiled Unit grows from 136 to 168
bytes and the parser continuation union from 112 to 120 bytes. The runtime
Function, Value, ScriptObject and 184-byte machine frame are unchanged. These
are recorded target layouts and logical charges, not process-memory or speed
measurements. Final private controls confirm the production environment, Unit
and frame layouts; the cumulative startup witness includes both the 32-byte
environment addition and the 32-byte empty class-record container. Author work
stays at 100,000 units, trusted initialization at 1,000,000 and cumulative heap
accounting at 8 MiB.

Final validation passes **80 public groups / 164 mode attempts**, **16 private
groups** covering 114 measured helper/parser operations plus one raw/layout
observation, and **two browser witnesses**. The direct Page and confined-worker
checks cover disabled, initial, repeated and click states at two viewport sizes
with exact pixels. The feature build also checks native raster-plan pixels;
this checkpoint did not execute hardware Vulkan.

The complete default Rust 1.88 suite passes **3,344 groups** and the Rust 1.99
`test262-host,vulkan-raster` suite passes **3,698**, preserving their complete
prior rosters plus 98 new groups. Strict Clippy checks for both configurations
and formatting pass. Rebuilt ordinary and host release adapters replay all eight
existing profiles: **1,330 case rows / 640 healthy controls**, with **1,222
passed, 60 failed and 48 unsupported**. Twenty formerly unsupported modes now
pass. Ten other modes become failures: eight reach unimplemented BigInt literal
syntax, and two `finally` subclass modes reach the existing final-unhandled-
rejection host audit. No previously passing case regressed. All raw changes are
retained; profile policies, original corpora and baselines are unchanged.

The retained attempts distinguish fixes from completed validation:

- The original 79 public groups passed in 162 mode attempts after the compiler
  found a sibling-visibility error. Widening `constructed_prototype` to
  `pub(super)` preserved its body and tariff. A separate private fixture compiler
  error required only a longer-lived expected-value binding.
- Later source review found that a bound nonconstructor superclass could enter
  generic argument forwarding before rejection. A class-only validation helper
  now charges one handle copy plus the existing constructor-check work after
  explicit arguments, or before default forwarding. One independent public group
  and one private group cover this correction; the original public expectations
  remain unchanged.
- The first full attempt had 2,993 passing and nine failing library groups:
  eight outer-sandbox socket refusals and one missed cumulative startup ledger.
  The ledger gained the derived 64 bytes described above, preserving its earlier
  terms, work and object counts. The final suites use that correction; the
  successful default rerun was allowed to create its required local sockets.

The final source receipt covers 1,834 selected inputs and 30 changed or new
paths. It is a cumulative selected inventory, not a complete build graph. No
Cargo/build inputs or conformance policies, corpora or baselines changed. These
results do not establish complete ECMAScript/web compatibility, production
security, process-memory use or a Chromium performance comparison.

The [evidence manifest](evidence/classes/manifest.json) and
[source/check archive](evidence/classes/sources-and-checks.tar.gz) retain the
source bindings, independent contracts, corrections and complete run records.
