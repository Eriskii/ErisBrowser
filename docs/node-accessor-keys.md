# Node accessor metadata keys

The getters and setters for `Node.prototype.nodeValue` and `textContent` now
reuse the existing immutable UTF-16 storage for their `length` and `name`
property keys. All four functions retain separate callable identities and
metadata objects. Their descriptors, property order and saved-accessor behavior
remain unchanged; replacing one function's name or length affects only that
function.

The Node installer borrows the two keys from the existing DOM interface pool.
It pays eight work units for the handoff and two units per factory before
retaining either key handle. Each factory keeps the existing object-slot guard,
display-string allocation, metadata insertion order and final Native allocation.
The original factory remains unchanged for other Node operations. Refusal can
retain an already charged private metadata prefix, but returns no callable.

Avoiding four duplicate key pairs saves 128 startup work units and 672 bytes of
accounted heap charges. Raw initialization now uses 99,867 of the unchanged
100,000-unit allowance, leaving 133; public Runtime constructors still reset the
author work allowance to 100,000. The focused Rust 1.88 default-feature run reports
1,738,688 accounted heap bytes, 749 objects in 749 reserved slots, 299 native
registry entries and 25 prototype entries. These are resource-policy charges,
not processor instructions, allocation residency or browser-performance results.

Eight new groups cover actual shared string backing, independent function
identities, public descriptors and saved behavior in strict and non-strict
modes, exact and one-short work/heap cuts, staged refusal prefixes, the one-time
handoff, the unchanged factory and the eager object-slot guard. On the parent,
the public behavior control passed both modes while the backing check failed
at the `getNodeValue` length key. The six private-API groups were introduced
with the implementation. All 37 focused Node groups, including the eight new
groups, pass with zero failures or ignored tests. Six existing test files have
explicit aggregate startup-accounting or label/comment migrations.

The complete Rust 1.88 default suite passes 2,895 tests. Rust 1.99 with
`test262-host,vulkan-raster` passes 3,249. Both runs include ignored tests and
report zero failures or ignored cases; strict Clippy passes in both configurations,
and Rust 1.88 formatting checks pass. The [evidence summary](evidence/node-accessor-keys/summary.json)
binds the source, frozen controls, parent failure, reviews and completed logs.
This validation does not establish a new GPU-device or browser-speed result.

This checkpoint adds no web API or persistent cache and leaves resource limits
and Runtime storage unchanged.
