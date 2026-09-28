# Ordinary addition conversion

The custom interpreter evaluates both operand expressions, then converts the
saved left operand followed by the saved right operand to primitives. Ordinary
objects use the default number hint: read valueOf live, invoke it with the
original receiver if callable, and read/invoke toString only when needed. The
first primitive result is retained. Noncallable hooks are skipped; exhaustion
of both hooks throws TypeError. Getter and call exceptions preserve identity.

If either resulting primitive is a string, both become strings and their
UTF-16 code units are concatenated. Otherwise the primitive numeric values are
added. This fixes boxed strings and objects returning strings without repeating
hooks. Signed zero, nonfinite values and ordinary Boolean/null/undefined
conversion retain their numeric behavior. Date's special default hint and
Symbol.toPrimitive, Symbol and BigInt remain unimplemented.

A chain converts each left-associative addition before evaluating the next
operand. Compound += uses the existing saved reference: target and key once,
Get, right expression, left conversion, right conversion, Put. A conversion
failure prevents the write; readonly write failure occurs after conversion.
Constant folding remains restricted to literal string operands.

Conversion bypasses the old generic string scan so host work checks do not run
before author conversion merely because an input is already a string. Both
conversion hooks finish before concatenated-result size checks. Primitive
formatting is precharged, including the existing decimal formatter's bounded
scratch. Concatenation precharges copying work and simultaneous Vec/Rc storage
before fallible Vec reservation. Its 262,144-code-unit maximum and the shared
instruction, allocation and nesting ceilings are unchanged. Looping and recursive
hooks stop with uncatchable resource errors. This is bounded accounting, not
an allocator audit or security certification.

Seven focused Rust groups cover type selection, original receivers, live hook
replacement, expression/hook order, exact abrupt identity, compound references,
UTF-16 surrogates, allocation/work boundaries and uncatchable stops. Page and
confined-worker fixtures verify six green samples followed by six blue samples
through an event callback. Existing scripts and rendering references are kept.

The [complete pinned addition directory](test262-addition.md) records remaining
Date and unsupported-prerequisite cases. The separate
[compound-assignment comparison](test262-compound-assignment-addition.json)
records ten newly passing += modes without changing their sources or policy.

Primary algorithm:
[ApplyStringOrNumericBinaryOperator](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-applystringornumericbinaryoperator).
