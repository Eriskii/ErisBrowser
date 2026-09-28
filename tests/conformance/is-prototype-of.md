# Object.prototype.isPrototypeOf

The custom interpreter implements `Object.prototype.isPrototypeOf` over its
existing internal object prototype graph. The method tests its argument before
converting the receiver: a primitive or missing argument returns `false`, even
with a null or undefined receiver. For an object argument, a nullish receiver
throws TypeError and a supported primitive receiver is boxed. The argument itself
is excluded from its prototype chain; identity, rather than property contents,
determines membership. Direct and inherited links reflect earlier successful
prototype mutations.

The walk reads internal links without accessing author `prototype`, `constructor`,
`__proto__`, `valueOf` or `toString` properties. Replacing the public
`Object.getPrototypeOf` does not affect it. Extra argument expressions still run
under ordinary call evaluation, but their resulting values are not coerced by
this method. Saved aliases and `call`, `apply` and `bind` use the actual receiver.

The method is writable, nonenumerable and configurable on Object.prototype. Its
name is `isPrototypeOf` and its length is 1; both metadata properties are
nonwritable, nonenumerable and configurable. The function inherits from
Function.prototype, is not constructable, and has no own `prototype` property.
Deleting or replacing the installed property does not invalidate a saved alias.

Each invocation and each internal prototype read consume the shared runtime
instruction allowance. Traversal uses constant space and permits at most 96
internal prototype reads. A deeper chain or an internally cyclic graph produces
an uncatchable resource error. This bounded policy is not an ECMAScript depth
limit. Supported primitive boxing uses the existing charged allocator; primitive
arguments return before that allocation. Object-receiver traversal does not
allocate retained records or scan string/array contents. The existing
100,000-step, 8 MiB allocation and call/stack limits remain unchanged.

Ordinary objects, arrays, callable values and supported boxed primitives use the
runtime's existing prototype identities. Host objects retain the currently
implemented host prototype model. Proxy traps, cross-realm objects, Symbol and
BigInt boxing, complete host prototype hierarchies and immutable-prototype exotic
semantics remain outside this slice. This method does not add those capabilities.

Six focused Rust regression groups use the `is_prototype_of_` prefix. They cover
metadata/aliases, conversion order, internal identity and mutation, poisoned
properties, shared work, exact depth boundaries, low-heap boxing and a synthetic
cycle with valid arena records. One group executes five already-pinned function
tests unchanged in both modes using the upstream assertion harness. No test body
or feature policy is changed by these runtime tests. The separate upstream
profile and its measured results are maintained independently.

```sh
cargo test --locked --offline --lib script::tests::is_prototype_of
```

Primary algorithms: [Object.prototype.isPrototypeOf](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.prototype.isprototypeof),
[ToObject](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-toobject),
and [OrdinaryGetPrototypeOf](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-ordinarygetprototypeof).
