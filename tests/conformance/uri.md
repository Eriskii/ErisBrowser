# URI encoding and decoding

The custom interpreter implements encodeURI, encodeURIComponent, decodeURI and
decodeURIComponent over lossless UTF-16 strings. Each global is a stable native
function with length 1, standard name/length flags, a nonenumerable mutable
binding and no constructor behavior. Saved and bound aliases remain callable
through name changes and global replacement. General Window descriptor
redefinition/reflection remains incomplete.

All four functions first perform ordinary string-hint conversion, reading
hooks live with their original receiver and preserving exact abrupt values.
Receiver and extra argument contents are ignored after normal argument
expression evaluation. Missing input converts to "undefined". Symbol conversion
remains a separate unsupported prerequisite.

The allocation-free codec visitor handles at most one Unicode scalar or raw
code unit per iteration. Encoding preserves the ECMAScript unescaped ASCII set;
encodeURI additionally preserves ;/?:@&=+$,#. Other scalars become uppercase
UTF-8 percent octets. Unpaired surrogate input throws canonical URIError.
Decoding requires complete hex triples and strict UTF-8, rejecting overlong
forms, invalid continuations, surrogate scalars and values beyond U+10FFFF.
decodeURI preserves reserved escapes with their original hex case. Raw
non-percent UTF-16 units, including lone surrogates, remain unchanged during
decoding. Plus signs remain plus signs, and decoding is performed only once.

After author conversion, both linear codec passes are charged. The first pass
validates and measures exact output with bounded stack scratch. The second
fills an exactly reserved Vec after charging output work and simultaneous
Vec/Rc storage. Output is limited to the existing 262,144 code units; all shared
instruction, allocation and nesting ceilings remain unchanged. No partial
output becomes observable on failure. This accounting is not a security audit.

Two codec groups cover scalar boundaries, every isolated surrogate and malformed
UTF-8 vectors. Seven runtime groups cover hook order, abrupt/canonical errors,
metadata and aliases, reserved characters, exact work/storage boundaries,
conversion before limit rejection and uncatchable recursive/looping callbacks.
A preliminary test cleanup attempted unsupported Window descriptor redefinition;
it was changed to ordinary assignment without altering that separate runtime
gap. Page and confined-worker fixtures preserve six green then six blue samples.

A root-authored Python urllib.parse.quote oracle checks 414 inputs across ASCII,
scalar boundaries, every Unicode plane and mixed strings. It supplies 3,312
assertions in 104 strict/sloppy batches, all passing; the frozen old adapter
fails all guarded batches. A deliberate wrong result yields Test262Error.
This is supplemental bounded coverage, not independent-agent review or a
replacement for complete upstream loops. Its input digest is
`9b8a2b0c77d1d1857b403cb6743da139e5f68841e322365e25dc2ed5b92700ac`;
local records are `artifacts/uri-matrix*.json`.

The [complete upstream profile](test262-uri.md) retains instruction-limit
outcomes and missing labeled-statement, reflection and Reflect prerequisites.
Primary algorithms:
[ECMAScript URI functions, Encode and Decode](https://tc39.es/ecma262/multipage/global-object.html#sec-uri-handling-functions).
