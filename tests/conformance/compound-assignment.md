# Compound bitwise assignment

The custom lexer and parser now recognize <<=, >>=, >>>=, &=, ^= and |=.
Longest-match tokenization places the longer shift assignments before their
shift prefixes. These use the existing right-associative assignment evaluator.

The target object and computed property key are evaluated once. Its current
value is read before the right-hand expression executes. Numeric conversion
then visits the saved left value and right value in order; only successful
conversion and arithmetic proceed to the write. The expression returns the
computed value. A right-hand side that changes which object a target-producing
function would return cannot redirect the saved reference.

Existing Number bitwise operations retain ToInt32 conversion, masked low-five
shift counts, signed right-shift behavior and unsigned right-shift results up
to 2^32-1. Fractions truncate, nonfinite values convert to zero, and the
operation's integer zero is positive. Conversion reads hooks live, so a left
hook may replace the right object's conversion hook before it is invoked.

Ordinary accessors, inherited receivers, const/TDZ/missing bindings and strict
versus sloppy write failures retain the reference evaluator's behavior. A
getter failure prevents right evaluation; a right-expression failure prevents
conversion; a conversion failure prevents the setter. Setter effects before
its own exception persist. Invalid targets and separated operator tokens are
rejected before script side effects. No new recursion, allocation strategy,
resource ceiling or quota reset is introduced.

Seven focused groups cover these stages, integer boundaries, right association,
line breaks, early rejection, shared loop/stack termination and the unchanged
Test262 decimalToHexString.js helper. One preliminary resource test incorrectly
relied on unsupported String.repeat; it was replaced with a recursive Number
conversion hook. The corrected test directly verifies uncatchable stack limits
without modifying runtime limits. Page and confined-worker fixtures exercise
all six assignments, exact pixels and reference order before and after a click.

The [complete pinned directory](test262-compound-assignment.md) includes other
arithmetic assignments, private fields and existing unsupported prerequisites.
This increment does not implement BigInt, Symbol, dynamic eval, super/private
references, classes or complete arithmetic coercion. The later
[logical-assignment increment](logical-assignment.md) adds the three short-circuit
forms separately. Loading
the hexadecimal helper enables its dependent parsing tests to reach their
full loops; any subsequent resource stop remains a nonpassing observation.

Primary algorithms:
[assignment evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-assignment-operators-runtime-semantics-evaluation),
[binary numeric application](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-applystringornumericbinaryoperator).
