# Unicode identifiers and lexical boundaries

The custom interpreter scans identifier names using the pinned Unicode 18.0.0
`ID_Start` and `ID_Continue` properties plus ECMAScript's identifier additions.
It does not use Rust Alphabetic/Alphanumeric or Unicode XID as substitutes.
The generated table, exact official data, license and byte hashes are recorded
in [the Unicode manifest](../upstream/unicode/18.0.0/manifest.json). The source
`DerivedCoreProperties.txt` SHA-256 is
`09c928886a178fcafd93c29e4bd59073a058e5a100b716d425cb563ab50f68c9`.
The offline generator is `tools/generate_js_identifiers.py`; table lookups use
an ASCII fast path and two indexed reads from deduplicated packed bitmap pages,
with no loop or per-query allocation. The same exact Unicode property data is
retained. The lexer reuses an already-validated raw first scalar instead of
querying its membership twice.

Literal, fixed-width `\uXXXX` and braced `\u{...}` spellings produce the same
identifier when they contribute the same code-point sequence. Braced escapes
accept bounded runs of leading zeroes. Invalid syntax, out-of-range values,
surrogates and characters invalid at the start/continuation position produce
parse SyntaxError. Two surrogate escapes are not combined into an identifier
scalar. Supplementary scalars can be written literally or with a braced escape.
No Unicode normalization, case folding or confusable-name merging occurs.

Tokens share immutable decoded name storage and retain whether their source
contained an escape. Literal grammar terminals require unescaped spelling,
including keyword operators, booleans/null/this, and object accessor or async
introducers. Escaping a reserved word does not make it a valid binding or
reference. Property IdentifierNames after a dot or in object keys can use those
spellings; shorthand properties still undergo reference-name validation.
Colon-form `__proto__` uses the decoded property name even when escaped.

Supported Script contexts distinguish ordinary `async`, `await`, sloppy `yield`
and sloppy `let` names from grammar introducers. Strict binding/assignment
restrictions also apply to decoded `eval`, `arguments` and restricted names.
Duplicate names compare after decoding, including function parameters and
lexical declarations. This does not add async functions, generators, modules,
classes/private fields, labels, destructuring or eval. Raw async/generator
introducers retain explicit UnsupportedFeature outcomes. Full early-error
validation inside unavailable grammars is not implemented; raw Script-goal
import/export still enters the existing unsupported-feature gate rather than a
complete module/goal validator. Unicode/escaped RegExp capture names remain
explicitly unsupported by the separate pattern parser.

Numeric literal adjacency uses the identifier-start rule instead of allowing
tokens such as `3in` to split into a number and operator. Existing numeric
grammar limitations remain. RegExp literal flags scan raw identifier-part
characters, and escaped flag letters are not accepted. The pattern parser's
existing flag validation and unsupported `u`/`v` behavior remain unchanged.
Provisional lexer syntax errors inside RegExp/template text can still be
discarded by the parser's appropriate rescan; resource errors terminate
immediately. Source offsets continue to count original UTF-8 bytes.

Whitespace uses the existing exact ECMAScript BMP set, accepting U+FEFF and
rejecting U+0085/U+180E. Only LF, CR, U+2028 and U+2029 create line-terminator
boundaries. Escape decoding never manufactures whitespace or comments.
Strings/templates retain their own escape rules.

Initial lexing and all suffix rescans share the compiler's existing 100,000-step
and 8 MiB cumulative allocation ledger. Token buffers, identifier decode
storage, AST-name/property-key copies, quoted-token storage and token movement
are charged before the associated operations. Token/name reservations are
checked; discarded suffix storage does not refund the ledger. The separate
256 KiB source, 32,768-token, parser-depth and 32-times-source lexical-rescan
ceilings remain. Accounting may terminate inputs below those size ceilings;
no quota was increased to admit identifier tests. Parse failures cannot create
runtime declarations. Existing execution and callback budgets still apply.
The parser borrows source only for its lifetime; retained code and strings own
their data. Provisional lexical diagnostics use precharged shared storage,
keeping ordinary tokens small; suffix rescans adjust only newly created,
uniquely owned diagnostics. These reduce actual work/allocation while preserving
the existing limits and cumulative accounting.
The private 20,000-π identifier input that previously exhausted binary-search
work remains an explicit successful optimization control; a 50,001-π input
still terminates on the same work limit. This does not alter any upstream test
source, policy or expected outcome.

Sixteen `identifiers_` groups cover literal/escaped aliasing, ID/XID differences,
marks/join controls/supplementary characters, normalization distinctions,
malformed escapes, reserved property names, raw accessor and contextual
introducers, duplicates/strictness, numeric/RegExp/template boundaries, exact
whitespace, shared diagnostic offsets, retained code after source disposal,
compact token allocation and private work/storage controls. The largest raw
Unicode declaration table that passed before this increment is also exercised
unchanged in both modes. Three existing pinned function
sources (`S13_A7_T1.js`, `S14_A5_T1.js`, `S14_A5_T2.js`) execute unchanged with
the upstream harness in both modes. These six cases do not define or filter a
new upstream inventory.

```sh
cargo test --locked --offline --lib script::tests::identifiers
```

Primary references: [ECMAScript names and keywords](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-names-and-keywords),
[identifier early errors](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-identifiers-static-semantics-early-errors),
[Unicode 18.0.0 official data](https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt)
and [Unicode Character Database format](https://www.unicode.org/reports/tr44/).
