# Short-circuit logical assignment

The custom lexer and parser recognize &&=, ||= and ??= as whole tokens and
require the existing simple identifier/member assignment targets. Evaluation
captures the reference and reads its value once. &&= returns a falsy old value,
||= returns a truthy old value, and ??= returns any old value other than null or
undefined. These paths do not evaluate the right expression or perform a write.
Objects are truthy without running conversion hooks. The exact old value,
including negative zero or object identity, is returned.

On the taken path, the right expression runs once and its value is written
through the saved reference. The expression returns that exact right value.
Computed keys and inherited accessor receivers remain stable when the RHS
changes which object a target-producing function would return. Getters, key
conversion, right expressions and setters preserve their exception identity
and prior effects; a failure prevents later stages.

Readonly properties and const bindings can be read successfully on a skipped
path. A taken path evaluates its RHS before applying existing strict/sloppy
property-write rules or throwing for const. Missing and uninitialized bindings
fail during the initial read. Assignment associates to the right and retains
the existing expression precedence and line-terminator handling. Invalid targets,
strict eval/arguments references and split operator tokens fail before execution.

An anonymous function or arrow directly on the RHS receives the identifier
name when the taken assignment target is an identifier. Parentheses retain this
behavior; sequence expressions, property targets and explicitly named functions
do not receive a replacement name. Inferred names preserve UTF-16 units and
standard nonwritable, nonenumerable, configurable metadata. Conversion storage
and scanning work are charged before name allocation. Existing allocation,
instruction and nesting limits are unchanged.

Eight focused Rust groups cover the truthiness/nullish lattice, original
receivers, single keys, skipped writes, abrupt stages, names, precedence, early
errors and resource behavior. A direct AST test places allocation accounting at
the existing ceiling: skipping an allocating RHS succeeds, while taking it
stops. It also checks the exact short-path instruction boundary. Recursive and
looping RHS callbacks stop with uncatchable limits and do not run the setter.
Page and confined-worker fixtures exercise all three operators and function
naming before and after a click, preserving six green then six blue samples.

The [complete upstream inventory](test262-logical-assignment.md) keeps classes,
private references, BigInt and other prerequisites explicit. This increment
does not implement those features or broaden earlier profile feature policies.
It does not establish full JavaScript compatibility or security.

Primary algorithms:
[assignment early errors and evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-assignment-operators).
