# Shared own-key enumeration accounting

Shared enumeration snapshots numeric keys by integer rank and other stored names
by creation order. Stored nonnumeric names are already unique in the object's
property map and order list, so the snapshot clones their shared handles without
repeating full-name deduplication. Virtual Array/String indices and `length`
retain precedence over overlapping stored properties. Already ordered snapshots
avoid sorting; disorder uses a charged in-place heap sort.

The path prepays metadata scans, virtual index formatting/encoding, dense-hole
searches, both snapshot buffers and caller result buffers. Numeric ranks avoid
sorting retained UTF-16 names. Array length shrink uses this shared path and no
longer prepays the removed full-name deduplication step.

For-in now groups visited names by UTF-16 length. An integer tree selects the
reached bucket; its inner tree compares full names only against names of that
length. Cached entries survive the live own-descriptor check without author
callbacks. Missing properties leave both trees unchanged; present nonenumerable
properties still shadow inherited names. Snapshot order controls enumeration,
independently of bucket order. Searches, insertion movement and cumulative node
storage consume the existing script budgets, including the extra outer-tree
headers. No quota is raised. The [length-bucket follow-up](for-in-length-buckets.md)
records the new validation; the results below describe the original checkpoint.

The [local fixture](own-keys.js) contains 43 sources, 86 sloppy/strict modes and
24 controls. Its expectations were frozen before implementation or engine
execution. It covers numeric/creation order, virtual keys, sparse arrays, UTF-16
names, descriptor collection, live reads and prototype traversal. Three mutation
cases (six modes) preserve current for-in behavior; they do not claim a universally
required result after prototype mutation. A six-block page fixture checks these
paths during document loading and click callbacks, including the confined worker.

The first candidate preserved all 17,822 ordinary historical case observations,
but four Number-statics controls reached the unchanged instruction limit. Their
sources, complete observations, source archive and binaries remain retained.
Each control repeated eight property checks whose for-in traversals searched
the same visited-name tree twice for each present key. Reusing the first search
removes that duplicate operation while retaining the live descriptor check.
The original corrected path uses 72,751 of 100,000 work units for either positive control;
the deliberate wrong assertions use 73,221 units and throw Test262Error. These
are instruction-accounting observations, not elapsed-time benchmarks. The initial
private regression and a subsequent import/test-assertion compilation failure
are retained in the [validation record](own-keys-integration-validation.json).

The [before](own-keys-initial.json) and [final](own-keys-final.json) local reports
have identical complete observations for all 86 cases and 24 controls. All
**39 historical profiles, 17,822 case observations and 3,900 controls** also remain
identical, and all **32 existing regression gates** pass without baseline changes.
Four selected older local suites preserve another 1,106 case observations and
148 controls. Existing failures, exclusions and resource outcomes remain visible;
this accounting change adds no conformance passes.

Rust 1.88 and 1.98 each pass strict all-target Clippy and **1,208 default / 1,219
Vulkan-feature tests**, with zero failures or ignored tests. Both release variants
preserve all **57 CPU pixel references**. The page fixture passes directly and
through the real confined renderer. Both HTML adapter binaries remain byte-identical
to the previous release, so no additional HTML replay is assigned. One deterministic
mutation-smoke run exercises 15,000 generated inputs without caught panics or
invariant failures; seventeen bounded paint stops remain within the checked
invariants. The unchanged root Python tooling retains its previous 244-test check.
See the [combined evidence](own-keys.json) and [unchanged limits](own-keys-limits.json).

This work covers shared key snapshots and reached for-in visited-name operations.
General live descriptor/map/Get/prototype/mapped-binding/JSON accounting and
allocator fallibility remain separate work. Tree node precharges are a cumulative
storage bound, not a fallible allocator API. Window, symbol-key and integrity
snapshots use separate unchanged paths. It establishes neither complete web
compatibility, production security nor a Chromium performance comparison.
Browser painting at that checkpoint remained on the CPU. The later optional
[native Vulkan route](../../docs/vulkan-native-window.md) has separate evidence.
