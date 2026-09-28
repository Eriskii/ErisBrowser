# Identifier default parameters

The custom parser and interpreter implement identifier default parameters in
ordinary function declarations and expressions, arrow functions, concise object
methods, and setters. Identifier [rest parameters](rest-parameters.md) are covered
by a subsequent increment. Destructured bindings, async/generator execution,
direct/indirect `eval`, `super`, `new.target`, classes, and dynamic `Function`
construction remain unsupported. This implementation supplies no replacement
for an upstream assertion harness and does not rewrite test bodies.

## Initialization and scope

All formal bindings exist before initialization. Defaults run from left to
right only when the corresponding argument is absent or `undefined`; `null`,
`false`, zero, and empty strings remain supplied values. Later bindings and the
binding being initialized are in the temporal dead zone, including reads via
`typeof` and writes. Initializer closures can observe bindings initialized
later, if called after initialization. An abrupt initializer stops subsequent
defaults and the body.

Functions with defaults have a parameter environment and a separate body
environment. Body `var`, function, and lexical declarations are invisible to
closures created in initializers. A body `var` matching a parameter starts with
that parameter's value; later body assignments to the separate variable do not
alter the captured parameter. Without a body declaration, assignments continue
to reach the parameter binding. This separation also applies to arrows and
methods. The implementation uses the existing environment arena, with no new
host capability.

Applicable `arguments` objects exist before defaults run, contain the actual
arguments and actual argument count, and are unmapped whenever defaults occur,
including in sloppy code. Their restricted `callee` accessor throws. Changing
an argument property does not replace a formal value or vice versa. An explicit
parameter named `arguments` suppresses that implicit binding and participates
in the TDZ. Body declarations named `arguments` do not suppress the object used
by initializers. Arrows retain lexical `arguments` and `this`; ordinary calls
normalize their receiver before evaluating defaults. Existing simple sloppy
parameter lists retain their mapped-arguments behavior.

Function `length` counts parameters before the first initializer. Anonymous
function and arrow expressions directly used as defaults receive the parameter
name; named functions and references to existing functions retain their names.
Object methods remain nonconstructable, and setter defaults retain setter
receiver behavior.

## Parsing and limits

Initializers use AssignmentExpression grammar with `in` enabled. Comma
expressions need parentheses. Nested arrows, object literals, regular-expression
literals, template literals, strings, and comments use the existing expression
parser. Parenthesized arrow cover forms are parsed once, then interpreted as
bindings when followed by `=>`; the parser does not rewind a token stream that
has undergone template or regular-expression rescanning. Trailing commas in
supported formal lists are accepted; invalid grouped binding targets and line
terminators before `=>` are rejected.

Non-simple lists reject duplicate names even in sloppy ordinary functions.
Arrows and methods continue to require unique parameters. Strictness inherited
from surrounding code applies to initializers and bodies. A function's own
raw `"use strict"` directive is an early SyntaxError with non-simple parameters,
including when the surrounding code was already strict. Parameter/body lexical
name conflicts remain early errors. Recognized invalid identifier-rest forms,
such as an initializer, a following comma, or a member target, produce
SyntaxError; valid identifier rest is now implemented separately. This is not
a complete validation implementation for unsupported binding-pattern grammar.

The existing 256 KiB source, 32,768-token, 96-depth parser, shared weighted
runtime stack, and call limits remain in place. Parameter metadata and shared
initializer holders are charged to the compilation allowance. Name-set
validation charges work and storage before allocation and avoids a quadratic
scan of formals for every body lexical name. Closure and call copies retain
shared initializer syntax while charging owned parameter/name metadata.
Defaults, body initialization, and callbacks share the normal 100,000-step
execution allowance and 8 MiB cumulative runtime allocation allowance; no
initializer resets them. Resource termination remains uncatchable, and the
body does not run after failed initialization.

## Evidence and remaining harness blockers

Eight focused Rust groups cover supplied values and evaluation order, TDZ and
deferred closures, body environment separation, arguments aliasing and shadows,
receivers/constructors/name/length/accessors, parser rescanning, early errors,
and depth/work/allocation failure. Run them with:

```sh
cargo test --locked --offline --lib script::tests::default_parameters
```

The full script suite contains 147 passing tests at this increment. Upstream
functions measurements and the unchanged case inventory are recorded
separately by the functions-profile runner; these local tests do not assign
upstream conformance credit or remove unsupported cases from its denominator.

The unchanged WPT `resources/testharness.js` inspected here is revision
`f085a1efc1f58fbe263d384b1e335d656fe58e66`, 198,291 bytes, SHA-256
`d2399236c2a09c429804ff2299ad6629e17e2b53f17a74dd341e936adb11ae3e`.
At the default-parameter checkpoint `ed7dcb8`, its former first blocker, `promise_setup(func, properties={})` on line 1141,
parsed. The next parser blocker was the rest parameter in `assert_wrapper`
on line 1504. Later code also needs rest parameters, destructuring, `for...of`,
and async/await, as well as host bindings. The [rest increment](rest-parameters.md)
records the later parser result. Default parameters alone do not make that
unchanged harness executable and does not establish a WPT pass.

Primary references: [FunctionDeclarationInstantiation](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-functiondeclarationinstantiation),
[function grammar and early errors](https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#sec-function-definitions-static-semantics-early-errors),
[ExpectedArgumentCount](https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#sec-static-semantics-expectedargumentcount),
and [IteratorBindingInitialization](https://tc39.es/ecma262/multipage/syntax-directed-operations.html#sec-runtime-semantics-iteratorbindinginitialization).
The unchanged WPT source is [testharness.js at its pinned revision](https://github.com/web-platform-tests/wpt/blob/f085a1efc1f58fbe263d384b1e335d656fe58e66/resources/testharness.js).
