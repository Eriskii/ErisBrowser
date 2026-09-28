# Ordinary relational conversion

The custom interpreter now converts both saved operands to primitives before
selecting relational string or numeric ordering. Operand expressions execute
first; ordinary number-hint conversion then visits source-left followed by
source-right. Live valueOf/toString lookups use the original receiver and keep
the first primitive result. A left hook can replace the right hook before use.
Noncallable hooks are skipped; failure to produce a primitive throws TypeError.

If both resulting primitives are Strings, comparison uses UTF-16 code-unit
ordering, including prefixes, surrogate pairs and isolated surrogates. This
fixes boxed strings and ordinary objects that return strings. It does not use
locale, normalized Unicode or scalar-value order. Mixed string/number operands
still receive numeric conversion. Numeric NaN is unordered and therefore yields
false for all four operators, including <= and >=. Signed zero compares equal;
nonfinite values retain numeric ordering.

The > and <= forms reverse comparison operands only after source-order author
conversion. The < and >= forms retain operand order. Inclusive comparison tests
for a defined false less-than result, preserving unordered NaN behavior.
Left-associative chains complete their earlier comparison before evaluating
the next expression. Getter/call exceptions preserve exact identity and prevent
later stages; captured operands are not replaced by later binding mutations.

The former addition-only primitive helper is shared under a number-hint name;
ordinary objects use this same hint for addition's default conversion. The
separate Number conversion helper and its diagnostics remain unchanged.
Relational string work checks occur after both primitive conversions and before
content access. Existing primitive numeric parsing keeps its work and temporary
storage charges. No allocation, instruction, string or nesting ceiling changes.
Recursive or looping hooks remain uncatchable resource stops.

Seven focused groups cover type selection, live conversion and expression
order, abrupt identity, UTF-16 and numeric boundaries, exact string-work and
numeric-storage boundaries, conversion before large-string rejection, and shared
callback limits. The existing script suite passes. Page and confined-worker
fixtures check six green samples followed by six blue samples after a click.

The [complete pinned comparison inventory](test262-relational.md) retains
BigInt/Symbol and dynamic-eval prerequisites. Date/Symbol.toPrimitive and broader
exotic conversions remain unsupported. This increment does not establish full
relational or ECMAScript conformance.

Primary algorithms:
[IsLessThan](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-islessthan),
[relational expression evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-relational-operators-runtime-semantics-evaluation).
