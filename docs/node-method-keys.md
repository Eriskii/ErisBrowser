# Shared Node method metadata keys

Nine additional Node functions now reuse the existing immutable `length` and
`name` keys: the `isConnected` getter, `getRootNode`, `hasChildNodes`, `normalize`,
`cloneNode`, `isEqualNode`, `isSameNode`, `compareDocumentPosition` and `contains`.
Their Native values, metadata objects, descriptors and display-name values remain
separate. Redefining or deleting one function's properties does not change another
function or a saved callable. This adds no API or language coverage.

Seven existing installers pass the borrowed key pair at their original call
positions. Each pays an eight-unit handoff before its existing four-unit entry
charge. The identity-method installer passes the pair to its three original
factories. Each factory retains its two-unit key-clone charge and all metadata,
owner insertion and Native construction charges. The four previously shared Node
accessors and ProcessingInstruction paths remain unchanged.

Removing nine fresh key pairs saves 324 work units. The eighteen retained clones
and seven handoffs cost 74, for a net saving of **250 work units and 1,512 accounted
heap bytes**. Actual raw startup now consumes **99,725 work units**, leaving 275
under the unchanged 100,000 limit, and accounts for **1,740,043 heap bytes**.
The object count and capacity remain 750; native registry entries remain 299,
prototypes 25 and global bindings 212. These are accounting observations, not
measured speed, resident-memory or native-stack improvements.

The existing factory still refuses a full object arena before publishing a bag.
Late refusals can retain already charged private metadata but cannot return a
partial callable. Owner insertion remains separately funded, and actual vector
capacity controls its existing growth charge. Work, heap, call and stack limits
are unchanged. The old fresh-key factory remains only under `cfg(test)` to retain
its independent 127/126-work and private-payload controls.

Eight new test groups cover all nine bags and eighteen strict/sloppy public rows,
plus 102 private attempts at exact and insufficient work/heap boundaries. The
first parent run exposed an error in the test's raw prototype-order expectation:
it omitted the leading `Symbol.toStringTag`. The source-derived correction
explicitly checks that symbol before the eleven string members. On the corrected
parent, all bag/descriptor checks and public rows pass before the first intended
sharing assertion fails. Both observations and the original test remain recorded.

All eight groups now pass. The complete suites pass **2,953 tests on Rust 1.88**
and **3,307 with `test262-host,vulkan-raster` on Rust 1.99**, each across 49 result
rows with no failed, ignored or filtered tests. Strict Clippy passes for all
targets in both configurations. Seven narrow existing-test migrations update
aggregate startup expectations or wording; isolated prior fees remain exact.
The changes add no GPU-device measurement or Chromium performance comparison.

A release adapter built from this source preserves both pinned Promise profiles:
`Promise.all` remains 170 passed, 18 failed and 8 unsupported across 196 modes;
the older methods profile remains 420 passed, 8 failed and 30 unsupported across
458 modes. All 90 and 74 harness controls pass respectively, with zero regressions
or newly passing cases. Every recorded case identity and status remains unchanged.

The [evidence record](evidence/node-method-keys/summary.json) retains the frozen
controls, original test defect, corrected parent observation, source reviews,
accounting ledger and complete validation bindings.
