# Flat regular-expression group parsing

The custom regexp parser and its two-unit prefix analysis now use explicit heap
frames. Capturing, noncapturing and lookahead group syntax no longer recurses on
the native stack. Capture numbering follows opening-parenthesis order, and each
quantifier retains the capture range it must clear on the next repetition.
Matching still uses the existing backtracking VM; nested lookahead execution
retains its separate native recursion guard.

The old grammar-depth and 128-capture limits are removed. Pattern length (8,192
UTF-16 units), node count (4,096), repetition count, shared work and cumulative
heap budgets remain unchanged. These bounds limit possible groups and captures;
VM capture storage remains charged before use. New frame/list backing allocations
are charged before reservation, including replacement allocations, without refunds
on failure or release. This is not a claim of complete allocation accounting.

Tests parse, analyze, execute and release patterns with 2,000 nested capturing
and noncapturing groups on a 128 KiB thread stack. Separate checks preserve
malformed-pattern errors, node/pattern/work/heap exhaustion, named and numeric
backreferences, repeated-capture clearing, and the lookahead execution guard.

## Preserved inputs and results

Against the frozen `1c660e8` release, the unchanged Test262 constructor-directory
inventory gains four passes: both modes of `S15.10.2.8_A3_T15.js` and
`S15.10.2.8_A3_T16.js`, which construct 200 nested groups. It now records **772
passed, 200 unsupported and four resource stops**. Every other observation in
the combined **28 profiles / 9,078 modes** is identical, as are all **2,288
assertion controls**, source fingerprints and runner policies. The 23 healthy
baseline gates are unchanged. The constructor profile remains an observation
inventory because XML-pattern matching and a full UTF-16 script loop still
exhaust their shared work budgets.

The [local fixture](regexp-deep-groups.js) was frozen and measured before changing
the engine. All **20 modes** change from resource stops to passes, with identical
fixture and mode fingerprints. A generated comparison against the preserved
parser also checks **3,888 identical parse outcomes** and **106,040 identical
full capture/error outcomes**, including sticky/scanning matches, two start
indices and case-insensitive multiline flags. This checks preservation of prior
behavior; it is not an independent conformance oracle.

Rust 1.88 and 1.95 pass strict all-target Clippy and **1,020 default / 1,031
Vulkan-feature tests**, none ignored. Both release builds preserve **57 CPU
pixel references**. **174 Python tests**, **45,000 deterministic mutation cases**
and unchanged HTML observations pass. [Machine-readable evidence](regexp-deep-groups.json)
records hashes and comparison results. No independent agent review, GPU exercise
or Chromium performance measurement was performed.

Unicode regexp syntax, broader legacy grammar, other String symbol protocols
and further matching performance work remain incomplete. This checkpoint does
not establish full web compatibility or audited security.
