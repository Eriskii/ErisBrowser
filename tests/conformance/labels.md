# Ordinary labeled control flow

Labels now target ordinary statements, including blocks and supported loops.
A named break exits its matching statement; a named continue begins the next
iteration only when that label directly names a while, do, for or for-in loop.
Consecutive labels share the same target. A label on a block containing a loop
is not a valid continue target. Nested loops and switches propagate jumps for
outer targets. Bare break and continue retain their existing nearest-loop/switch
rules, including for updates, do conditions and per-iteration lexical bindings.

The parser resolves decoded label identifiers to numeric targets. Duplicate active
names, unknown targets, nonloop continue targets and jumps across function
boundaries are SyntaxErrors. Ordinary functions, methods, arrows and parameter
initializer functions isolate label scopes. Labels can be reused after a scope
ends or in a nested function. Strict eval/arguments labels remain allowed;
strict reserved words are rejected. Break/continue respect automatic semicolon
insertion after line terminators, including those inside comments.

Statement-only grammar rejects lexical/class declarations but permits sloppy
`let` as an expression followed by a newline, except for the prohibited `let [`
lookahead. The original upstream ASI case exposed this distinction; it and the
block/array variants remain unchanged.

Labels add no binding scope. Var-name collection and declaration hoisting visit
their bodies, including unreachable declarations, and lexical conflicts remain
early errors. Try/finally runs on named jumps and can replace them with another
jump, return or throw. Exact thrown values are retained. Host resource stops
remain uncatchable and do not run cleanup author code.

Active names are capped by the existing MAX_DEPTH. Identifier copies, temporary
record storage and bounded name comparisons are charged before allocation or
content scans. Consecutive aliases are parsed iteratively into one AST wrapper;
runtime jumps carry only numeric targets, without repeated string lookups or
allocation. A labeled parser entry counts three depth units, accounting for its
two extra retained helper frames. Existing resource ceilings are unchanged.

A new deep-label regression initially caused a debug-thread stack overflow.
Accounting for helper frames alone did not resolve it. Splitting the large
statement dispatcher into nine existing keyword parsers reduced its local debug
frame from 40,216 to 6,344 bytes. Those sizes describe this compiler/build only.
The failing regression now returns a resource error. Another regression checks
excessive nesting of blocks, if, while, for, do, try/finally, switch and function
declarations without native stack exhaustion. No thread stack or global quota
was enlarged.

Nine focused groups cover nested targets, loop kinds and iteration closures,
finally overrides, hoists/function scope, early errors, Unicode and ASI, parser
work/storage bounds, deep statement rejection and uncatchable loops. Page and
confined-worker fixtures check six green samples followed by six blue samples
after a click. Local failure/prologue evidence is retained under
`artifacts/labels-stack-probe.log`, `artifacts/labels-parser-frame-disassembly.log`
and `artifacts/labels-statement-stack-final.log`. The final dispatcher prologue is
recorded in `artifacts/labels-parser-final-frame.log`.

The [complete pinned inventory](test262-labels.md) retains all original sources
and metadata. Legacy sloppy labeled function declarations, with, for-of,
async/generator/module syntax and tail calls remain incomplete. Strict labeled
functions are syntax errors. General statement completion-value accounting,
including full UpdateEmpty behavior, remains a separate gap; eval-based cases
remain subject to the existing dynamic-eval limitation. This is not complete
label, ECMAScript or web-platform conformance, or proof of security.

Primary algorithms:
[labelled statements](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-labelled-statements),
[break](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-break-statement),
[continue](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-continue-statement),
[LoopContinues](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-loopcontinues).
