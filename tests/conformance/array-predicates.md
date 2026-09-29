# Array every and some

`Array.prototype.every` and `some` now use live property operations on arrays
and supported ordinary array-like receivers. They capture length once, skip
absent entries, include inherited values and stop at the first decisive callback
result. Receiver conversion and length access precede callback validation, even
for empty inputs. Callback arguments, `thisArg`, mutations and abrupt values use
the existing interpreter call machinery; neither method creates a result array.

These independently authored local fixtures cover **41 semantic groups**, paired
across `Array.prototype.every` and `some`: **82 source cases / 164 strict and
sloppy variants**. Each `// CASE:` block in
[array-predicates.js](array-predicates.js) executes in a fresh runtime with
unchanged pinned Test262 assertion helpers. Eight separate controls check both
successful assertions and deliberate SameValue/error-type failures.

The source and expectations were frozen before candidate implementation. The
published release at commit `f5a5a85586c759835b4e4701052684cfaa2db989` records
**164 failed semantic variants**. The candidate passes all **164 unchanged
variants**, and all **8 assertion controls** behave as expected in both runs.
Every semantic case checks method presence and true/false results before its
other assertions, preventing missing-method TypeErrors from producing false
passes. The complete observations remain in the
[machine-readable evidence](array-predicates.json). The separate
[complete upstream inventory](test262-array-predicates.md) retains its failures,
unsupported modes and resource stops; it has no healthy baseline gate.

The before adapter SHA-256 is
`44ec604be831b65d08caaa7a94aae65437339967cda2948eb5c5feac57cdd0ee`.
The final adapter SHA-256 is
`c5cb23f3fdb36a848937fdb2a8853987a91b2ea874dfec2a15b525b94eec04b4`.
The unchanged fixture SHA-256 is
`7a82528b49633c55c9d82dfb70693ea017960d459789414e3977348039a2d124`;
the original matrix SHA-256 is
`2e5b95009375bcce2483cd27edc95491885ad8d405f813b94cb82802578baf15`.
The evidence embeds the frozen matrix, preparation provenance, release
metadata, exact case sources/modes, expectations, harness hashes and results.

Expectations follow the current primary algorithms for
[every](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.every)
and [some](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.some).
Coverage includes conversion/callback-validation order, empty results,
truthiness and short-circuiting, holes and inherited indices, live getters and
mutations within captured length, callback arguments and receivers, abrupt
identity, ordinary argument evaluation, symbols, metadata and saved aliases.
Poisoned hooks check that result conversion, constructors and iterators are not
consulted. Large logical lengths return after one callback, including a sparse
u32 array with two leading holes.

Replay must retain each case's source bytes, mode, metadata, helper order and
expected result. The original report's `file` label is `array-predicates.js`;
its published path is `tests/conformance/array-predicates.js`. That display-path
mapping is outside the case fingerprint and does not authorize changing any
hashed input. Strict mode remains an adapter mode, not a source rewrite.

Local replay uses a three-second limit per mode, bounded captured output, a
512 MiB address-space cap, inherited 30-second process CPU limit and 45-second
outer runner deadline. Runtime tests additionally cover callback-allocation
refusal after getter effects, recursive length/index/callback paths, terminal
work exhaustion and frame cleanup. Early-return allocation does not grow with
logical length; scanning a large sparse range can still exhaust shared work.
The existing work, cumulative allocation and recursion limits are unchanged.

Proxy, BigInt, typed arrays, general host array-like receivers and cross-realm
behavior remain outside this implementation's verified scope. No GPU exercise,
security certification, Chromium performance comparison or full conformance
claim is made.
