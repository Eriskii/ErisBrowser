# TypedArray relative indexing and searches

Number TypedArrays implement `at`, `includes`, `indexOf`, and `lastIndexOf`
through the engine's own interpreter and buffer storage. All ten Number element
kinds use the same methods. BigInt views and shared memory remain incomplete.

The methods validate the receiver and capture its length before converting an
index. Later buffer growth does not extend that captured range. A conversion
callback may shrink or detach the buffer; subsequent reads check the current
storage. `includes` can match the resulting `undefined` values, while the index
searches skip absent elements. `at` returns `undefined` for an absent element.
Initially detached or out-of-bounds views throw before index conversion.

`includes` matches NaN; the two index searches do not. All searches treat the two
zero signs as equal and leave the search value unconverted. `lastIndexOf`
distinguishes an omitted starting index from an explicit `undefined`. Empty
searches skip index conversion, while `at` still converts its argument.

These behaviors follow the ECMAScript algorithms for
[`at`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.at),
[`includes`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.includes),
[`indexOf`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.indexof),
and [`lastIndexOf`](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-%typedarray%.prototype.lastindexof).

Open the interactive example with:

```sh
./run.sh ./examples/typedarray-search.html
```

## Validation

The [pinned upstream report](../tests/conformance/test262-typedarray-search-after.json)
retains 145 original Test262 files and all 290 strict/non-strict modes:

| Result | Modes |
| --- | ---: |
| Passed | 78 |
| Resource limit | 78 |
| Failed | 2 |
| Unsupported | 132 |

All 56 harness controls pass, including six paired positive/wrong-result probes.
The resource outcomes make the complete report unhealthy for baseline recording;
it is not an upstream CI gate. The exclusions retain 88 BigInt, 26 complete-helper
and 18 host-hook modes. The two failures call the unimplemented `TypedArray.fill`
before `lastIndexOf`; the original test and failure remain visible. The
[before report](../tests/conformance/test262-typedarray-search-before.json) used
the published views engine and records only 12 passes, with unhealthy controls.
Those earlier passes alone did not establish method support.

The full Rust 1.88 suite with `vulkan-raster` passes 2,310 tests. All 1,305 script
tests pass on Rust 1.88 and 1.98. Fourteen new search groups cover all ten codecs,
equality and index boundaries, ordered callbacks, changing buffer state, saved
methods, terminal resource limits and cleanup. Existing metadata tests check the
new method identities, descriptors and exact installation costs. The Page test
checks literal displayed lines and result-area pixels before and after a click;
it runs the direct Page path rather than the confined worker.
Strict Clippy checks pass on both toolchains, and all 293 Test262 tooling tests
pass. The Rust groups and Page example run in the existing CI suite.

Adding four prototype members initially pushed the existing nonnumeric-descriptor
upstream regression over the unchanged 100,000-work limit. Two optimizations
remove actual repeated work: sufficiently large ordinary objects whose keys
cannot be array indices copy creation-order names directly into the result, and
canonical numeric-index classification rejects impossible first units before
parsing. Neither caches mutable property or buffer state. Five new classifier
and seven new snapshot groups cover exact and one-short work/storage limits and
fallback behavior. Both modes of the retained descriptor test now pass with
6,401 work units remaining. The initial failed runs remain in the evidence.

The older TypedArray foundation and view reports retain their case outcomes:
590/34/488/126 and 46/0/96/64 passed/failed/unsupported/resource respectively.
All 46 local view cases and 24 paired controls also pass. No resource quota,
original test body, existing baseline or old profile policy was relaxed.
Across those suites and 38 historical profiles, all 17,842 prior case rows and
4,072 control rows remain exact, with no lost passes or policy changes.

The [evidence summary](evidence/typedarray-search.json) binds the candidate
engine, source, reports and checks. Its [archive](evidence/typedarray-search.tar.gz)
retains original inputs, source reviews, prior comparison reports and failed
development runs alongside successful results.

The complete browser compatibility, security and Chromium performance goals
remain unfulfilled.
