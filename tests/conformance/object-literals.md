# Object literal implementation and focused checks

The custom parser and interpreter support ordinary object literals with literal
and computed string property names, identifier shorthand, concise methods,
getters/setters, and the special static `__proto__` colon form. This is a bounded
subset of ECMAScript object initializers, not a complete language claim. No
external JavaScript engine, rewritten fixture or native assertion substitute
supplies these semantics.

Computed names use AssignmentExpression with `in` permitted. A comma needs
parentheses: `{[(a,b)]: value}` is valid, while `{[a,b]: value}` is a SyntaxError.
Each key expression runs once, followed immediately by string-hint primitive
conversion, before its value expression or method creation. For supported
objects this retrieves/calls `toString`, then `valueOf` if necessary. Abrupt
completion stops later keys and values. Arrays and functions honor their own
conversion methods. Numeric keys use the existing ECMAScript decimal string
conversion; property keys preserve UTF-16 code units, including lone surrogates.

Methods and accessors capture the surrounding lexical environment, receive the
call's receiver, and inherit strictness or enable it with their own directive.
Object methods are not inherently strict. They are not constructors and have no
own `prototype` property. Their `name` is the evaluated key, with `get ` or `set `
for accessors; `length` counts parameters before the first initializer. Both
properties are non-writable, non-enumerable and configurable. Anonymous function
and arrow expressions in data-property values receive the property name;
explicitly named functions and references to existing functions keep their
names. Names are stored as UTF-16, independently of lexical self-binding names.

Data properties are writable, enumerable and configurable. Getter/setter pairs
merge without losing the opposite accessor; a later data property or accessor
performs the corresponding descriptor transition. Duplicate ordinary keys
retain ordinary property ordering and overwrite semantics. Shorthand requires
an IdentifierReference, and accessor introducers require the actual `get`/`set`
identifier token. Quoted property names do not become shorthand or introducers.
Methods reject duplicate parameters even in sloppy code; getters require zero
parameters and setters one.

A static identifier or string-literal key whose cooked value is `__proto__`,
followed by a colon, evaluates its value and sets the new object's prototype if
the value is an object or null. Other values are evaluated and ignored. This
creates no own property, performs no value coercion and does not infer a name
for an anonymous function value. Duplicate static prototype setters are an early
SyntaxError, including differently escaped string-literal spellings. Computed,
shorthand, method and accessor forms of `__proto__` are ordinary properties.
JSON.parse retains its separate own-data-property behavior.

The existing 256 KiB source limit, 32,768-token limit and 96-depth parser guard
apply. Object entry storage is charged conservatively before vector growth to
the shared 8 MiB compilation allowance, and entries consume compilation work.
Runtime coercion and property operations share the 100,000-step allowance,
8 MiB cumulative allocation allowance and weighted stack guard. Accessor-name
prefix copying is charged before allocation and obeys the 262,144-code-unit
string limit. Prototype walks share the 96-link guard and consume work before
mutation. Resource termination remains uncatchable.
Owned function parameter/name copies are preflighted before closure retention
and temporary call copies; function bodies remain shared.

Run the six focused regression groups with:

```sh
cargo test --locked --offline --lib script::tests::object_literals
```

The groups cover key/coercion/value order and exceptions, primitive and custom
conversion keys, property ordering, computed methods/accessors and receiver
strictness, name/length descriptors, nonconstructability, UTF-16 names,
anonymous-name inference, descriptor merging, prototype forms and parse-time
errors, and hostile coercion, key sizes, prototype chains and nesting.

Symbols and `Symbol.toPrimitive`, object spread, async/generator methods,
`super`/home-object behavior, and rest/destructured parameter lists remain
unsupported. Unsupported method forms are not replaced with ordinary methods.
Identifier defaults now share the [default-parameter implementation](default-parameters.md),
including separate initialization scope and unmapped arguments.
Identifier Unicode escapes and complete function source reflection are also
outside the current parser/runtime subset. The separate bounded
[Array reversal and Number radix increment](array-number-methods.md) supplies
the remaining methods used by the existing pinned JSON ASCII-escaping test.

No new upstream selection or conformance credit is assigned by these local
regressions. The unchanged JSON/string, RegExp and template-literal inventories
remain independent gates.

Primary references: [ECMAScript object initializer grammar, early errors and
evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-object-initializer)
and [method definitions and accessor evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#sec-method-definitions).
