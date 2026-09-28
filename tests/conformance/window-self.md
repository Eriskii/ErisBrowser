# Replaceable Window.self

The browser runtime initializes `self` as an own Window accessor with enumerable
and configurable set to true. Its stable getter and setter have names `get self`
and `set self`, lengths 0 and 1, and the usual nonwritable, nonenumerable,
configurable function metadata. Both inherit Function.prototype and are
nonconstructable. The getter returns the private Window identity. The setter
replaces the property with the supplied value, without converting it, as writable,
enumerable and configurable data. Null or undefined accessor receivers select
the realm Window; other receivers, including an object inheriting from Window,
throw TypeError.

Bare `self` and members of the realm Window consult the same property record.
Initially this includes `window.self` and `globalThis.self`; replacing globalThis
changes what that public alias refers to, independently of the self property.
Author data/accessor redefinitions use ordinary descriptor compatibility checks.
Descriptor queries report fresh records without invoking getters. Saved native
accessors remain callable after replacement or deletion. A saved Replaceable
setter throws if the current nonconfigurable property prevents its full data
property definition, even when called by sloppy code. Other readonly writes and
failed deletions retain the usual strict versus sloppy behavior.

Deleting the own property leaves it absent. It does not uncover a hidden original
accessor on Window.prototype. A saved descriptor can restore it, or a new runtime
starts with the original accessor. Supported author-created inherited `self`
properties participate in both member and bare-name lookup. Inherited accessors
receive Window for bare access, while a bare call to an authored function value
still supplies an undefined receiver before the callee's strict/sloppy receiver
normalization. Captured references recheck the live property after RHS side
effects; a strict unresolved identifier does not become resolved because the RHS
creates a property.

Global lexical bindings remain in a separate environment. `let self` can shadow
the bare identifier without changing Window.self; deletion or replacement of the
property does not delete or overwrite that lexical binding. An uninitialized
`var self;` preserves an existing accessor, while its initializer uses assignment.
If the property is absent, a fresh global var declaration creates a
nonconfigurable data property. Under the current ECMAScript global declaration
algorithm, an earlier var declaration over an existing configurable property does
not itself prohibit a later script's lexical declaration. Existing lexical
conflicts and nonconfigurable-property restrictions still apply.

A global `function self` is checked before that script inserts any lexical, var
or function bindings. An incompatible nonconfigurable property causes TypeError;
earlier lexical-name conflicts cause SyntaxError. Failed declaration preflight
leaves earlier names absent for later scripts. A compatible declaration defines
the function as a nonconfigurable writable enumerable data property. These checks
do not run author accessors.

The shared `Object.defineProperty` entry point also uses the existing bounded
string-hint ToPropertyKey conversion for ordinary and supported host objects.
It rejects a primitive target before key conversion, converts the key before
reading the descriptor, observes array/function overrides, preserves abrupt
completions and retains exact UTF-16 keys. This does not add Symbol keys. Host
names containing NUL or lone surrogates do not alias `self`.

The accessor pair and enlarged binding records are charged to the existing
8 MiB cumulative allocation allowance before they are retained. A failed
allocation while defining an accessor leaves the prior self property intact.
The cached self key avoids repeated UTF-8 key allocations in the global property
lookup. Prototype traversal consumes shared work and is capped at 96 reads;
accessor calls and reentrant mutation use the existing shared call/stack guards.
The 100,000-step budget and uncatchable resource termination remain unchanged.
Global declaration preflight uses the same bounded traversal as var hoisting,
without inserting records. Resource termination is not a transactional rollback
of earlier successful author effects.

This is a scoped Window binding correction. General Window definitions,
reflection such as host `hasOwnProperty`/`propertyIsEnumerable`, extensibility,
WindowProxy behavior, cross-origin/cross-realm objects, named child properties
and a complete Window prototype hierarchy remain unsupported. The subsequent
[global-value correction](global-values.md) extends the shared property handling
to globalThis, undefined, NaN and Infinity, including their descriptor flags.
Definitions outside those four names and self retain the existing unsupported
result. Window/document data descriptors still have known conformance gaps.
The core ECMAScript adapter still shares the browser host realm; these changes
do not separate it into a host-free realm or claim full Window WPT coverage.

Nine `window_self_` Rust regression groups cover both execution modes, accessor
metadata/brands, saved descriptors, locking, inherited lookup, captured references,
reentrant getters/setters, separate-script declarations and lexical shadows,
declaration rejection without residual bindings, key conversion order/identity,
private allocation/work failure and a synthetic cycle with valid arena records.
One group runs the already-pinned function case `13.2-30-s.js` unchanged through
the upstream harness in both modes. Corpus bytes and runner policy are unchanged.

```sh
cargo test --locked --offline --lib script::tests::window_self
```

Primary sources: [HTML Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#the-window-object),
[Web IDL Global](https://webidl.spec.whatwg.org/#Global),
[Replaceable](https://webidl.spec.whatwg.org/#Replaceable),
[attribute bindings](https://webidl.spec.whatwg.org/#es-attributes),
[GlobalDeclarationInstantiation](https://tc39.es/ecma262/multipage/ecmascript-language-scripts-and-modules.html#sec-globaldeclarationinstantiation),
[global environment records](https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-global-environment-records),
and [Object.defineProperty](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.defineproperty).
