# Decimal Number formatting

The exact binary64 value `1000000000000000.25` previously became
`"1000000000000000.3"`. It now becomes `"1000000000000000.2"`: both shortest
decimal candidates are equally close, and ECMAScript requires the even decimal
significand. The same formatter serves decimal Number methods, primitive string
conversion, addition, JSON, property conversion and numeric literal keys in
ordinary and dynamically compiled functions.

The implementation retains Rust's shortest digit generation and the existing
ECMAScript notation boundaries. It changes a candidate only after proving an
exact midpoint with integer arithmetic. For decimal `s * 10^q` and normalized
binary `M * 2^E`, a tie with an adjacent candidate requires
`E == q - 1` and `(2*s ± 1) * 5^q == M`. Negative powers require exact division.
An even significand needs no correction. Checked arithmetic and early bounds
limit each candidate to 24 divisions or 22 multiplications.

Each reached correction stage first pays its work charge. Rewriting the decimal
significand also reserves 32 bytes before allocating. Runtime and compiler
budgets use the same fallible entry point, so refusal cannot publish a formatted
property name. Prior author callbacks retain their effects when later formatting
or compilation exhausts its allowance. Existing quotas and formatting charges
remain in place. These are logical resource charges, not machine instruction
counts or a proof of allocator overhead.

The 74 frozen raw-bit fixtures cover both signs, midpoint neighbors, notation
boundaries, nonfinite values, signed zero and finite/subnormal extremes. Their
expected strings are independently checked by enumerating exact rational
binary64 rounding intervals:

```sh
python3 tools/verify_number_format.py
cargo test --lib script::number_format
cargo test --test number_format_page -- --include-ignored
```

The private tests also exercise exact and one-short work/storage budgets,
callback effects, literal keys, strict/sloppy execution and deliberately wrong
controls. The [example page](../examples/number-format.html) displays the amount,
its JSON representation and a property key; clicking **Change sign** updates
the retained Text nodes. Direct Page and confined-worker tests compare the
complete canvas with independent, script-free literal HTML before and after
the click.

The [validation record](evidence/number-format.json) binds the changed source,
test commands, retained failures and before/after adapter reports. Across five
unchanged upstream profiles, all 1,163 mode results are preserved: 933 pass,
226 remain unsupported and four existing Symbol failures remain. The new local
midpoint cases establish the correction separately from those profiles.

This correction relies on Rust's shortest/minimal-nearest conversion contract;
it does not independently reimplement or prove that complete algorithm. Other
ECMAScript gaps, nondecimal fractional formatting, full web compatibility,
production security and the Chromium performance target remain open.

Reference: [ECMA-262 Number::toString](https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-tostring).
