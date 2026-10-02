# DataView over nonshared ArrayBuffer storage

Eris now implements the DataView constructor, `buffer`, `byteOffset`,
`byteLength`, authentic `ArrayBuffer.isView`, and the nine Number getter/setter
pairs: Int8, Uint8, Int16, Uint16, Int32, Uint32, Float16, Float32 and Float64.
Fixed and length-tracking views share the original buffer's private storage.
Index and value conversions run in specification order; fresh bounds are
checked after callbacks. Detached or out-of-bounds views retain their buffer
identity. Ordinary indexed properties and `Object.freeze` do not freeze bytes.

The implementation follows the [DataView algorithms](https://tc39.es/ecma262/multipage/structured-data.html#sec-dataview-objects).
Float16 encoding rounds directly from binary64 with ties to even, avoiding an
intermediate binary32 rounding. Public tests require valid NaN classification
and stable encoding without imposing a particular payload. BigInt codecs,
shared buffers, typed arrays, Proxy, foreign realms and immutable buffers remain
gaps. These results do not establish complete web compatibility, production
security or the requested Chromium performance threshold.

## Independent inputs and complete observations

The [preparation evidence](../../docs/evidence/data-view-preparation/index.json)
preserves the pre-implementation checkpoint: the complete 561-source upstream
subtree, 1,122 modes, seven original helpers, independent local byte literals,
paired controls, review corrections and the original before observations.
The pinned revision is `7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd`.
The original upstream and local fixture sources, harnesses, exclusion rules and
resource policies remain unchanged.
The [runtime result record](data-view-runtime.json) binds the final observations,
validation and baseline publication.

| Population | Final observed result |
| --- | --- |
| Formal DataView, 1,122 modes | 694 passed, 12 failed, 416 unsupported |
| Formal DataView controls | 232/232 verified |
| Local DataView, 138 modes | 124 passed, eight failed, two unsupported, four expected resource outcomes |
| Local DataView expectations / controls | 128/138; 92/92 verified |
| Complete 43-profile replay | 19,685 cases and 4,484 controls; 696 gains, zero pass losses |
| Complete nine-suite local replay | 1,620 cases and 484 controls; 126 gains, zero pass losses |
| Explicit local resource expectations | All 40 matched; all 36 older records unchanged |

Six upstream BigInt-method sources lack the BigInt metadata tag and remain
admitted. Their twelve modes still fail: eight parse failures and four failed
nonconstructability assertions. No incidental pass is claimed as BigInt support.
The `Float16Array` metadata umbrella admits the DataView Float16 methods; it does
not imply a Float16Array constructor or `Math.f16round` implementation.

The only gains in older profiles are the two `Object.seal` DataView modes.
The older ArrayBuffer local fixture also gains its two DataView prerequisite
modes, reaching 92 passes, six failures, two exclusions and four expected resource
outcomes: 96/104 expectations verified. Every other old complete case/control
record is unchanged. All 34 old gates passed before baseline publication.
The new DataView gate and the two strengthened Object.seal baseline entries come
from exact projections of the completed reports, without another engine run.
There are now 35 known-state gates; reported failures and exclusions stay visible.

## Initialization regression and correction

The first implementation passed all 21 DataView private groups and the complete
external replays, but failed two existing direct-native string tests. They use
the instruction budget remaining after `Runtime::new`; DataView's generic
intrinsic installer consumed too much of that budget. Both failures and the
first candidate's original source and results remain retained.

The corrected installer constructs final function descriptors once, reuses
located vacant map entries and reserves the prototype's creation-order buffer
once. It removes repeated searches and rewrites while retaining explicit work
and storage charges. Search costs use actual runtime tree sizes. Existing tree
insertions prepay a conservative node allowance, and vector reservations pay
the full requested buffer plus relocation. No quota, budget boundary, old test
body or external oracle was changed. The measured runtime leaves 71,936 of 100,000 work units after initialization, with 582,570 bytes charged to the cumulative ledger. The original two failing tests both pass.

The accounting uses logical work units and requested cumulative storage, not
CPU instructions or allocator telemetry. General arena accounting and infallible
B-tree allocation remain broader limitations; see [security](../../docs/SECURITY.md).

## Validation

Rust 1.88 and 1.98 each pass **1,316 default and 1,433 native-feature tests**, including the Linux confinement checks. Strict all-target Clippy passes for default, `vulkan-presenter` and `vulkan-raster` on both toolchains; formatting passes. All 27 private DataView groups pass. The final adapter uses the successful Rust 1.98 default release.

Private tests include every binary16 decode word, signed finite adjacent
midpoints and their neighboring binary64 values, independent byte vectors,
callback effects and exact/one-short work/storage admission. Direct and confined
browser integration tests check literal pixels and text before and after a click,
including view state retained between document execution and the event callback.

The first browser integration fixture used `Object.is`, which is still absent,
and failed with a non-callable value. Its negative-zero expectation was retained
using the equivalent Number check `n === 0 && 1 / n === -Infinity`, alongside the
unchanged literal sign byte. The initial fixture, failure and reviewed correction
are retained separately. A reciprocal-only proposal was rejected before editing
because tiny negative nonzero values can also reciprocate to negative infinity.
The final candidate changes only that fixture from the second candidate; its
production inventory and reused release binaries are identical. Both full
validation matrices and external replays are bound to the final source archive.

The [runtime evidence inventory](../../docs/evidence/data-view-runtime/index.json)
binds all three candidates, failed and successful validation, all complete replay
reports, reviews and publication records. It contains source overlays and data,
not executable builds. The [preparation CI receipt](../../docs/evidence/data-view-preparation-ci.json)
retains all nine successful jobs for the preceding public checkpoint.

```sh
cargo test --locked --lib script::data_view::tests
cargo test --locked --test pipeline data_view
cargo test --locked --test worker data_view -- --include-ignored
cargo build --locked --release --bin eris-js
python3 tools/test262_conformance.py --profile data-view \
  --baseline tests/conformance/test262-data-view-current.json
```

The worker test requires Linux Landlock ABI 6. A passing known-state gate preserves
recorded passes; it does not mean every case or every web standard passes.
