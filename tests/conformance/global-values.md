# Tracked global value properties

The custom runtime initializes four ECMAScript global value properties with the
following live data descriptors. These flags govern real reads, writes, deletion
and definitions; descriptor queries do not substitute synthetic flags.

| Name | Initial value | Writable | Enumerable | Configurable |
| --- | --- | --- | --- | --- |
| globalThis | private realm Window identity | true | false | true |
| undefined | undefined | false | false | false |
| NaN | NaN | false | false | false |
| Infinity | positive infinity | false | false | false |

A fixed five-name selector covers these properties and the previously supported
Window.self. They use the same authoritative global binding records, descriptor
validation and final store. Other Window property definitions remain unsupported.
The existing self native accessor retains its own target: borrowing its descriptor
onto globalThis does not retarget its setter to globalThis.

Assignment to globalThis preserves an existing data property's flags. Author
accessor definitions, locking, restoration, deletion and ordinary assignment
recreation are supported. Recreating an absent property by assignment produces
writable, enumerable and configurable data. Bare names and member access follow
the same live own/inherited property, except that local or lexical bindings take
precedence for identifiers. Author getters and setters receive the actual property
receiver; a bare global accessor receives Window. An authored function called
through a bare identifier still follows normal callee receiver normalization.

Immutable global assignments silently fail in sloppy code and throw TypeError in
strict code. Their RHS and any required compound/update coercion still run.
Sloppy deletion returns false; strict member deletion throws TypeError and strict
identifier deletion remains an early error. SameValue-compatible definitions are
allowed, including another NaN value. Incompatible values, attributes or accessor
conversion throw TypeError without changing the property. Assigned or compared
values are not subjected to invented author coercion.

Deleting globalThis leaves it absent unless an author-created inherited property
exists. Bare and member lookup, typeof, `in`, deletion, inherited setters, readonly
inherited data and own shadowing are coherent within the existing prototype
model. Captured bare writes recheck live property existence after RHS or numeric
coercion. A strict unresolvable reference stays unresolvable even if its RHS creates
the property; strict member writes follow member-property rules. Accessor callbacks
may delete/redefine the property without a stale post-callback value being stored.

The private realm identity is independent of the public property. Replacing,
deleting or lexically shadowing globalThis does not alter top-level this, sloppy
function receiver normalization, the native self getter, document.defaultView or
Window event targets. Global lexical bindings remain in their own environment.
Local bindings may shadow all four names; restricted top-level lexical names
(undefined, NaN and Infinity) fail with SyntaxError. A configurable globalThis
property permits a top-level lexical binding without replacing that property.
Its own lexical TDZ still applies.

Redundant uninitialized var declarations preserve existing properties and flags.
After own deletion, a fresh global var declaration creates nonconfigurable,
enumerable, writable data, bypassing an inherited setter. The current ECMAScript
global declaration rules permit a later lexical declaration after var reused an
existing configurable property; there is no historical VarNames restriction.

For all five tracked names, global function declaration checks precede every
lexical/var/function insertion. Existing lexical conflicts take SyntaxError
precedence. Incompatible nonconfigurable properties cause TypeError with earlier
prospective bindings and function records still absent. The checks traverse
functions in reverse source order, and only the last declaration of each tracked
name creates a function value. Compatible declarations use the global-function
attributes and preserve permitted nonconfigurable writable enumerable data flags.
No author getter or setter runs during these declaration checks or definitions.
This is not a claim of complete Annex B or arbitrary host global declaration rules.

The implementation retains direct reads of existing own data bindings. Missing
or accessor-backed tracked globals use the existing charged prototype/call paths.
The fixed UTF-16 key cache is charged during initialization; exact fixed-name
matching and removal do not allocate repeated UTF-8 copies. Accessor pairs, new
keys and binding records are charged before the final store. A rejected final
allocation leaves the previous property or its absence intact. Shared limits remain
100,000 instructions, 8 MiB cumulative allocation, 96 prototype reads, and the
existing call/weighted-stack caps. Recursive accessors, internally cyclic valid
arena graphs and repeated descriptor/recreation operations cannot reset these
limits. Resource exhaustion remains uncatchable. Earlier successful author effects
are not rolled back when later work terminates.

Object.defineProperty retains the shared target-check, string-hint ToPropertyKey,
and descriptor-conversion ordering, including abrupt/reentrant callbacks. Near-miss
names, embedded NUL and isolated UTF-16 surrogates do not alias these globals.
Symbols, cross-realm objects, WindowProxy/cross-origin semantics, general Window
own-key enumeration, host hasOwnProperty/propertyIsEnumerable and extensibility
are outside this increment. Protected window/document behavior is unchanged.
The subsequent [Window reflection checkpoint](window-reflection.md) implements
own-key enumeration and membership/enumerability over supported Window bindings;
general descriptor definitions, extensibility and full Window semantics remain
incomplete. Symbols also have their own later implementation record.
The adapter still shares the browser host realm. This global-property increment
introduced no eval, Date or URI capability. The later [URI increment](uri.md)
adds four encoding/decoding functions; complete global-object conformance
remains unfulfilled.

Eleven `global_values_` Rust groups cover both execution modes, descriptor flags
and SameValue, assignment/deletion/coercion order, realm/event identity, reentrant
accessors, captured/inherited references, lexical/var/function separation,
all-five-name declaration atomicity and reverse-order/last-definition behavior,
exact property keys and private allocation/work/cycle boundaries. All earlier
Window.self groups remain part of the script regression suite. The separate
complete pinned upstream profile retains its unrelated dependencies and outcomes;
this implementation does not edit fixture bytes or runner policy.

```sh
cargo test --locked --offline --lib script::tests::global_values
```

Primary sources: [global value properties](https://tc39.es/ecma262/multipage/global-object.html#sec-value-properties-of-the-global-object),
[global environment records](https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-global-environment-records),
[GlobalDeclarationInstantiation](https://tc39.es/ecma262/multipage/ecmascript-language-scripts-and-modules.html#sec-globaldeclarationinstantiation),
[Object.defineProperty](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.defineproperty),
and [ValidateAndApplyPropertyDescriptor](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-validateandapplypropertydescriptor).
