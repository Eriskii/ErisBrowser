# Class `super` properties

Class constructors, public instance/static methods and accessors, and their
lexical arrows now support `super.name` and `super[key]`. The selected slice
includes reads, calls, simple/compound/logical assignments, prefix/postfix
updates, `new super.C(...)`, and assignment targets in `for-in` and `for-of`.
`delete super.x` is accepted syntax and throws ReferenceError when evaluated.
Ordinary nested functions do not inherit a class method's `super` context.
Nested class heritage and computed member names use their surrounding context;
the nested class's own methods receive their own home.

A function retains the identity of the object where its class member was
defined. Instance methods and constructors use the original class prototype;
static members use the original constructor. Borrowing or binding a method
does not replace that home. Each new reference reads the home's current
prototype, so later prototype changes remain observable. The receiver stays
separate: getters, setters and method calls receive the actual `this`, including
supported primitive receivers. An alternate `new.target` can choose an instance
prototype without replacing the constructor's home.

The [current SuperProperty algorithm](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-super-keyword)
checks initialized `this` before evaluating a computed key, then captures the
home's prototype. The raw key remains unconverted until the property read or
write. Simple assignment therefore evaluates its right-hand side before key
conversion. Compound assignments and updates convert the key once during the
read and reuse it for the write. Logical assignments omit the right-hand side
and write when their condition short-circuits. A null base refuses before key
conversion; simple assignment still evaluates its right-hand side first.

Calls preserve the reference receiver through parentheses; a comma expression
loses that reference. A throwing callee getter prevents argument effects, while
a noncallable result is rejected after argument evaluation. Deleting a super
reference evaluates its key expression but throws before key conversion or
property deletion. Escaped arrows retain the original home and lexical
activation, including an uninitialized derived receiver. Direct `super()`
continues to use the constructor's live parent independently of property lookup.

Receiver-aware lookup and writes reuse existing descriptor and exotic-object
routes. A writable base data property examines the receiver's own descriptor;
it does not call a setter found on the receiver's prototype. Base accessors
receive the real receiver. Array index/length definitions, TypedArray numeric
indices and conversions, mapped arguments, represented DOM properties and
native Promise brands retain their respective behavior. This does not enable
previously unsupported host reflection paths such as unrepresented DOM
properties, Style/ClassList reflection or Window event-handler descriptors.

Home attachment, owner capture and reference completion each have an explicit
eight-unit admission. Owner capture additionally pays the existing tick per
visited lexical environment. Raw-key retention and reference read/write/call
ownership are funded before dependent author effects; lookup and descriptor
operations retain their existing charges. The new receiver-aware wrappers pay
six units at entry, with one additional receiver-copy unit for an accessor get
or a set. They add no separate heap allowance; existing core allocation charges
still apply. Malformed internal owner/home/function or captured arena IDs refuse
before property dispatch.

Complete-shape probes on the selected x86_64 Linux configurations measured the
runtime Function growing from 104 to 120 bytes and Reference from 56 to 80.
The top execution frame remains 184 bytes and Environment remains 112.
Typed function-slot and growth accounting includes the added storage. These
are representation measurements and logical accounting, not process-memory or
speed measurements. Author work remains 100,000 units, trusted initialization
1,000,000 and cumulative heap accounting 8 MiB.

Public Resource and Unsupported failures preserve reached effects while
suppressing later bodies, catch/finally work, iterator closing and queued jobs.
Private helper refusal alone is not a public terminal boundary. The independent
public contract contains 65 groups in 130 planned sloppy/strict modes, including
mode-specific mapped-arguments expectations and eleven natural terminal sources.
Its literals were frozen independently of producer code. Separate private
contracts exercise home/owner/reference admissions and receiver routes.

Object-literal method/accessor `super`, fields, private elements, static blocks,
async/generator methods, optional chaining and tagged templates remain outside
this slice. Proxy objects, multiple realms and the other documented language
gaps remain unsupported. This is not complete class or ECMAScript conformance,
a security certification, or a performance comparison.

Final validation passes **3,432 tests with Rust 1.88 default features** and
**3,796 with Rust 1.99 and `test262-host,vulkan-raster`**, with no failures or
skipped tests. Both complete runs contain 55 result rows. Both final strict
Clippy checks and rustfmt pass. The new coverage comprises 65 public groups
covering 130 sloppy/strict modes, four parser groups, eight receiver-wrapper
groups, nine capture groups with 84 measured helper calls, and two browser
witnesses. Native-plan assertions in the full suite run on the CPU; this
checkpoint makes no new GPU correctness or browser-performance claim.

The retained first private run passed six groups and failed one: an out-of-range
internal Object ID passed a kind-only base check. Bounded arena-ID validation
now refuses it without changing the frozen oracle. The first complete Rust
attempt stopped after 3,085 library passes and three failures in historical
expectations. The migrations update the Function-size assertion, the
Phase/ExprFrame/Reference layout assertions, and the cumulative bootstrap heap
expression for the added home slot. They leave production behavior and the
selected tariffs unchanged. The final suites include those migrated tests.
The evidence also retains the parent public result (10 passes and 55 failures),
the parent browser failure, and the parent socket-fixture refusal followed by
its successful authorized rerun.

Fresh ordinary and host release binaries replay all eight existing profiles:
**1,330 cases comprise 1,258 passed, 64 failed and eight unsupported**. Every
complete case record and all **724 control records** match the published
`db02f8c` predecessor exactly, with no transitions, prior-pass regressions or
control changes. The comparison recheck produces the same summary. Existing
failed and unsupported outcomes remain; these bounded profiles do not establish
full Test262 conformance. Runner/importer/protocol code, feature policies,
corpora and baseline files are unchanged, and no baseline was rewritten.

The [evidence manifest](evidence/class-super-properties/manifest.json) and
[retained sources and checks](evidence/class-super-properties/sources-and-checks.tar.gz)
bind the final 28 changed source paths, validation attempts, release-binary
provenance and profile comparisons. The 15,289 selected input inventory includes
expanded unchanged source coverage; it is not a self-contained build graph.
