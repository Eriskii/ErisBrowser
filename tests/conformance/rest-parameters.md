# Identifier rest parameters

The custom interpreter accepts a final `...BindingIdentifier` in ordinary
function declarations and expressions, parenthesized arrows, and concise object
methods. Identifier defaults may precede it. Getters and setters reject rest
parameters. Destructured bindings, call/array/object spread, async/generator
execution, classes, `eval`, dynamic `Function` construction, `super`,
`new.target`, and custom iterator protocols remain outside this increment.

Each call creates a fresh intrinsic Array from the remaining actual argument
values. An empty remainder still creates an Array. Entries are dense own data
properties, including entries whose value is `undefined`; object references
retain identity. Rest creation does not call the global `Array` binding,
prototype setters, getters, or an author iterator. Changes to `arguments` in an
earlier default do not replace values in the original actual-argument list.

Parameter-list simplicity and initializer-expression presence are separate.
Identifier rest is non-simple but contains no expression. Consequently, rest
lists have unmapped arguments objects even in sloppy functions, restricted
`arguments.callee`, unique parameter names, and an early error for an own raw
`"use strict"` directive. Inherited strictness remains supported. Actual default
expressions continue to control parameter/body environment separation and
whether body declarations suppress an implicit arguments object; rest alone
does not introduce that separation. A rest parameter named `arguments`
suppresses the implicit binding. Arrows retain lexical arguments and receivers.

Earlier defaults see the rest binding in its temporal dead zone. A closure
created by a default can observe the Array after initialization. Default failure
prevents rest construction and body execution. With defaults present, a body
`var` matching the rest name starts with that Array but has a separate binding;
without defaults, the body redeclaration shares the parameter binding.

Function `length` stops before rest or an earlier initializer. Existing naming,
receiver normalization, method nonconstructability, ordinary construction, and
call/apply/bind behavior remain in use. Arrays produced here retain the
runtime's existing Array API and explicit descriptor limitations; this does
not implement every Array exotic-object operation.

## Syntax and resource boundaries

The rest parser requires one contiguous three-dot ellipsis, an identifier, and
the closing parenthesis. It rejects extra dots, a missing or invalid identifier,
an initializer, a trailing comma, a later formal, member access, duplicate
bindings, forbidden strict names, and parameter/body lexical conflicts.
Whitespace and comments between the ellipsis and its identifier are allowed.
The arrow cover parser consumes its prefix once, including nested templates and
regular-expression literals; a terminal rest item requires `=>`, without a
line terminator before it. Unsupported destructuring patterns are reported as
unsupported when recognized, rather than executed as identifier rest.

The new rest flag is included in parameter metadata accounting. Tail copying
charges work before scanning and validates copy, retained-vector, and ordinary
property-record allocation before changing array arenas. The vector uses a
checked exact reservation. Copy and retained storage are conservatively charged
separately, with no refund. Rest Arrays have the existing 65,536-element limit.
Even empty rest Arrays consume allocation. Repeated calls, defaults, and rest
construction share the existing 100,000-step execution allowance, cumulative
8 MiB allocation allowance, and weighted call/stack guards. Source, token,
parser depth, and rescan bounds are unchanged. Resource termination remains
uncatchable; failed initialization does not execute the body.

## Checks and provenance

Six focused regression groups cover fresh/dense arrays, property attributes,
intrinsic construction, arguments non-aliasing, scope/TDZ/default interactions,
receivers and metadata, call/apply/bind/construction, syntax restrictions,
unsupported patterns, and allocation/work failure before array retention.

```sh
cargo test --locked --offline --lib script::tests::rest_parameters
```

At source freeze all 153 script tests and strict library/tests Clippy pass. The
direct Page fixture also passes, including default/rest interactions and an
event callback with rest. Separate unchanged upstream inventories and their runners determine
conformance counts; local regression tests neither rewrite those sources nor
remove unsupported cases from their denominators.

The unchanged WPT `resources/testharness.js` at revision
`f085a1efc1f58fbe263d384b1e335d656fe58e66` has 198,291 bytes and SHA-256
`d2399236c2a09c429804ff2299ad6629e17e2b53f17a74dd341e936adb11ae3e`.
A full-source parse now passes the former `assert_wrapper(...args)` rest
blocker and reports an unsupported destructuring binding. The source contains
destructuring `for...of` on line 2550 and later async/await and host dependencies.
This is progress through parser prerequisites, not an executable unchanged WPT
harness or a WPT pass. No external JavaScript engine or native assertion shim
supplies these semantics.

Primary references: [parameter syntax and early errors](https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#sec-parameter-lists),
[ContainsExpression](https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#sec-static-semantics-containsexpression),
[IsSimpleParameterList](https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#sec-static-semantics-issimpleparameterlist),
[FunctionDeclarationInstantiation](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-functiondeclarationinstantiation),
and [IteratorBindingInitialization](https://tc39.es/ecma262/multipage/syntax-directed-operations.html#sec-runtime-semantics-iteratorbindinginitialization).
The inspected WPT source is [testharness.js at the pinned revision](https://github.com/web-platform-tests/wpt/blob/f085a1efc1f58fbe263d384b1e335d656fe58e66/resources/testharness.js).
