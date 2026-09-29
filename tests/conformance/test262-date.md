# Pinned Test262 Date inventory

This profile retains the **entire recursive Date subtree: 594 sources in 51
directories, 1,188 sloppy/strict modes**, at Test262 revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`. All 678,607 source bytes remain
unchanged. The [manifest](../upstream/test262-date/manifest.json) binds every
source/helper and six original Git API proof documents. Selection does not
filter sources by implementation support or expected outcomes.

The Date-only proof path follows the pinned commit/root through `test/built-ins`
to Date tree `6ad4fab73be4a87e6bfb58793b5fa9c82335ce5e`. It retains the original
recursive subtree response, reconstructs every descendant binary Git tree
hash, and requires the exact complete 51-directory set. A root-linked harness
tree authenticates all seven helpers; legal files are root-linked too. No
projected tree is represented as an original API response. The separate path
allows at most eight proof documents, four MiB of proof bytes, 4,096 recursive
entries, 64 directories, path depth eight and two MiB of source bytes. It keeps
the existing profiles' direct-tree path and limits intact.

| Directory | Sources | Modes |
| --- | ---: | ---: |
| `Date` | 78 | 156 |
| `Date/UTC` | 17 | 34 |
| `Date/now` | 6 | 12 |
| `Date/parse` | 8 | 16 |
| `Date/prototype` | 44 | 88 |
| `Date/prototype/Symbol.toPrimitive` | 18 | 36 |
| `Date/prototype/constructor` | 1 | 2 |
| `Date/prototype/getDate` | 8 | 16 |
| `Date/prototype/getDay` | 8 | 16 |
| `Date/prototype/getFullYear` | 8 | 16 |
| `Date/prototype/getHours` | 8 | 16 |
| `Date/prototype/getMilliseconds` | 8 | 16 |
| `Date/prototype/getMinutes` | 8 | 16 |
| `Date/prototype/getMonth` | 8 | 16 |
| `Date/prototype/getSeconds` | 8 | 16 |
| `Date/prototype/getTime` | 8 | 16 |
| `Date/prototype/getTimezoneOffset` | 8 | 16 |
| `Date/prototype/getUTCDate` | 8 | 16 |
| `Date/prototype/getUTCDay` | 8 | 16 |
| `Date/prototype/getUTCFullYear` | 8 | 16 |
| `Date/prototype/getUTCHours` | 8 | 16 |
| `Date/prototype/getUTCMilliseconds` | 8 | 16 |
| `Date/prototype/getUTCMinutes` | 8 | 16 |
| `Date/prototype/getUTCMonth` | 8 | 16 |
| `Date/prototype/getUTCSeconds` | 8 | 16 |
| `Date/prototype/setDate` | 14 | 28 |
| `Date/prototype/setFullYear` | 20 | 40 |
| `Date/prototype/setHours` | 23 | 46 |
| `Date/prototype/setMilliseconds` | 14 | 28 |
| `Date/prototype/setMinutes` | 18 | 36 |
| `Date/prototype/setMonth` | 17 | 34 |
| `Date/prototype/setSeconds` | 17 | 34 |
| `Date/prototype/setTime` | 11 | 22 |
| `Date/prototype/setUTCDate` | 7 | 14 |
| `Date/prototype/setUTCFullYear` | 6 | 12 |
| `Date/prototype/setUTCHours` | 11 | 22 |
| `Date/prototype/setUTCMilliseconds` | 8 | 16 |
| `Date/prototype/setUTCMinutes` | 8 | 16 |
| `Date/prototype/setUTCMonth` | 9 | 18 |
| `Date/prototype/setUTCSeconds` | 9 | 18 |
| `Date/prototype/toDateString` | 7 | 14 |
| `Date/prototype/toISOString` | 17 | 34 |
| `Date/prototype/toJSON` | 13 | 26 |
| `Date/prototype/toLocaleDateString` | 4 | 8 |
| `Date/prototype/toLocaleString` | 4 | 8 |
| `Date/prototype/toLocaleTimeString` | 4 | 8 |
| `Date/prototype/toString` | 8 | 16 |
| `Date/prototype/toTemporalInstant` | 8 | 16 |
| `Date/prototype/toTimeString` | 6 | 12 |
| `Date/prototype/toUTCString` | 9 | 18 |
| `Date/prototype/valueOf` | 6 | 12 |
| **Total** | **594** | **1,188** |

There are no frontmatter negatives, flags, locale requirements, fixture files
or non-JavaScript leaves in this subtree. Helpers are unchanged `assert.js`,
`sta.js`, `propertyHelper.js`, `compareArray.js`, `isConstructor.js`,
`assertRelativeDateMs.js` and `dateConstants.js`; the latter two are newly
required by this selection. LICENSE and INTERPRETING.md remain unchanged.
Manifest SHA-256:
`41195dabf14aba17fadbe6e1a9412f6ac70b660f6cd73508621874e2f94c19c2`.
Ordered source/mode/helper identity digest:
`d4d054267aa2d73510caa8b1594e4ee45f78ecef1a97c77d985cd598038a384e`.

The feature policy is exactly `CONSTRUCTION_FEATURES | REGEXP_FEATURES`,
unchanged from independent preparation. It admits 1,166 modes and retains
22 metadata exclusions: six cross-realm modes and sixteen Temporal modes
(including the BigInt-tagged cases). Untagged dependencies remain executable.
In particular, both modes of `Date/year-zero.js` and `Date/parse/year-zero.js`
retain their pre-existing for-of parser failure; they are not filtered as
unsupported metadata.

The **340 assertion controls** retain all 32 existing core controls and add
77 guarded positive/deliberately-wrong pairs in both modes. They check
construction, coercion, arithmetic, local offsets, ISO/JSON/Symbol behavior,
Annex B aliases and all standard method metadata. The selected method must
be available and successfully invoked before error assertions. Each mismatch
control additionally requires its separately executed positive partner to
verify; a missing Date producing incidental Test262Error in both partners
cannot establish assertion health. Method metadata uses the unchanged
propertyHelper with restoration enabled.

The [formal initial report](test262-date-initial.json) uses published before
adapter SHA-256
`68a36c3e0d2641c63c90d72a5596e503fa19ad30e2e751888113f4c28b58d977`:
**1,166 failed / 22 unsupported**, with no resource or process failures.
All 1,188 complete observations and fingerprints exactly match the prior
independent diagnostic. Only the 32 core controls verify; none of the 308
Date controls verify. No healthy baseline was recorded. The report SHA-256 is
`754413e6084d6a1757e85bde6546448e2e0ef3384b30b3b6c578e12e3eb0cf3d`.
The [first frozen candidate](test262-date-first-candidate.json) records
**1,162 passed / 4 failed / 22 unsupported**, with all 340 controls verified.
The four failures are the unchanged for-of parser prerequisites named above.
The [complete first comparison](test262-date-first-comparison.json) retains
all identities and observations: 1,162 gains, zero losses, 26 unchanged
outcomes. The [final exact-input release](test262-date-final.json), adapter
`ab40a96f2cc1908359186e7c0648cecddc5ad8f8b2adf8a60de0e0d2c9a8f220`,
reproduces every first-candidate case and control observation after the
accounting fixes. Its production input digest is
`7bb6ff9b49fa7a66f88cf803a9fec4aee3eb0c4b76d3e14bd5472900c2218a02`.

The [Date baseline](test262-date-current.json) is now recorded and the
[CLI gate](test262-date-gate.json) passes with zero regressions. This is a
**known-state gate with four retained failures and 22 exclusions**, not an
all-pass conformance claim. All 340 assertion controls verify and no Date
case has a resource/process failure. The initial unhealthy report and the
first candidate remain unchanged.

The [complete final comparison](test262-date-profiles-comparison.json) covers
all 36 profiles, **16,146 cases and 3,592 controls**. In the 35 older profiles,
130 former failures now pass; all other observations and every control outcome
remain unchanged. All 28 existing healthy baseline CLI gates pass. Only eight
baselines with Date gains were strengthened; twenty were left byte-identical.
The seven observation-only selections remain without baselines. The
[recording receipt](test262-date-baseline-recording.json) identifies each file.
No fixture bytes, policy, expectation, source/mode/helper identities or existing
script quota definitions changed to obtain these results.

All **35 earlier profile contracts / 14,958 cases / 3,252 controls** remain
byte-for-byte identical as captured records, including policies, exclusions,
source/helper/mode fingerprints, expected controls and prerequisite identity.
Their canonical SHA-256 remains
`e6a81c9fa4863df4f9797833c184f6ef212a547340ba758af160de44902f2574`.
The Date contract was frozen before candidate evaluation at
`ec461ebe97f6a34fcec4b0e6d19f959d30123c08ea4f75fdbdebb3f93ef34ddf`.

```sh
python3 tools/test262_conformance.py --profile date --binary target/release/eris-js --baseline tests/conformance/test262-date-current.json --output artifacts/test262-date-report.json
```

The environment and host timezone are not normalized to UTC. Helpers use
each instant's actual getTimezoneOffset where relevant. See the separate
[independent local Date fixtures](date.md) for captured non-UTC host facts.
This is a complete retained subtree and a bounded known-state measurement,
not a claim of complete Test262 or ECMAScript conformance.
