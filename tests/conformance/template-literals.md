# Template literal implementation and focused checks

The custom JavaScript parser implements untagged template literals, including
`${...}` substitutions, nested templates and multiline cooked text. This is a
bounded language subset, not full ECMAScript conformance. No external parser,
JavaScript engine or source-to-source transformation supplies this behavior.

The parser selects template lexical goals. A backtick suspends ordinary token
scanning; template text resumes after the expression parser recognizes its
closing substitution brace. Strings, comments, object/function bodies and
RegExp literals inside a substitution therefore use their ordinary grammars.
Substitutions accept comma expressions and permit the `in` operator even inside
a surrounding `for` initializer. Template text does not form a strict-mode
Directive Prologue.

Cooked segments preserve UTF-16 code units, including lone surrogates. Literal CR
and CRLF sequences become LF; LS and PS are retained. Escaped line continuations
contribute no units. Standard character, hexadecimal and Unicode escapes are
cooked, including braced Unicode escapes with leading zeros. Legacy decimal or
octal escapes and malformed hexadecimal/Unicode escapes produce parse-phase
SyntaxErrors for untagged templates, before any script statement runs. This
shares string escape decoding with ordinary quoted literals; fixed-width
hexadecimal escapes require actual hexadecimal digits.

Evaluation processes each substitution and immediately converts its result to a
string before evaluating the next. For supported objects this uses the string
hint: retrieve and call `toString`, then `valueOf` if necessary. Getters, method
calls, exceptions and custom array/function conversions remain observable.
Output assembly uses a flat sequence of segments and avoids repeatedly copying
the entire accumulated prefix. UTF-8 conversion occurs only at the existing
host/display boundary, not during template evaluation.

Source and token limits remain 256 KiB and 32,768 tokens. Template text scanning
shares the 100,000-step compile allowance and 8 MiB compile allocation allowance;
all cooked storage is charged before growth. Parser-directed rescanning shares
the existing 8 MiB cumulative source-span allowance with RegExp literals. One
template may have at most 4,096 substitutions, with the other limits potentially
terminating it earlier. Nested templates share the parser's 96-depth allowance.
Runtime coercion and copying share the 100,000-step execution allowance, 8 MiB
cumulative allocation allowance, weighted call-stack guard and 262,144-code-unit
string limit. Resource termination cannot be caught by script code.

Run the five focused template groups with:

```sh
cargo test --locked --offline --lib script::tests::template_
```

That filter also selects the existing DOM template-fragment test. The new groups
cover nested grammar boundaries, braces/backticks in comments/strings/RegExp,
comma expressions, escape cooking and line normalization, lone-surrogate JSON
round trips, coercion order and abrupt completion, strict-prologue separation,
malformed escapes, tagged-template classification, large output, deep nesting,
many substitutions and uncatchable coercion loops/recursion.

Tagged templates are explicitly unsupported, including raw arrays, frozen
call-site objects and their identity. Symbol/`Symbol.toPrimitive`, BigInt,
dynamic eval, generators and async expressions retain their existing unsupported
status. Other built-in conversion behavior remains limited by the implemented
runtime; for example, complete Function.prototype.toString is not supplied by
this increment. A mixed upstream file containing tagged and untagged checks is
retained as one unsupported file variant, not partially credited.

The separate, unchanged upstream selection and its complete measured outcomes
are documented in [the Test262 template profile](test262-template-literal.md).
Existing JSON/string and RegExp inventories are retained separately.

Primary references: [ECMAScript template lexical components and cooked values](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-template-literal-lexical-components)
and [template literal grammar, early errors and evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-template-literals).
