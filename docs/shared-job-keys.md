# Shared job metadata keys

`queueMicrotask` and the Promise installer now reuse the same immutable UTF-16
storage for their `length` and `name` property keys. Each function retains its
own metadata object, descriptors, values and callable identity. Replacing a
function's `name` or `length` does not change another function's properties.

The producer returns two owned string handles after publishing `queueMicrotask`.
The adjacent Promise installer moves those handles into its existing literal
pool. It still checks that its private state is empty before allocating Promise
metadata. No persistent cache, Runtime field or global is added. Startup errors
still prevent the partially initialized Runtime from being returned.

The producer prepays cleanup and two handle clones, increasing its charge from
327 to 331 work units. The shared consumer uses 994 rather than 1,032 units and
avoids 168 bytes of accounted heap charges for two duplicate strings. Combined
startup saves 34 work units: raw initialization uses 99,873 of the unchanged
100,000 allowance, leaving 127. These are resource-policy charges, not measured
processor instructions or allocator residency. The standalone installer fixture
keeps its original 1,032-unit boundary and allocation order.

Eight new test groups cover physical backing identity, independent public
properties and queued behavior, exact and one-short work/heap boundaries,
allocation prefixes, all six invalid installer states and partial initialization
after refusal. Before implementation, the physical-sharing test failed on the
parent's separate Promise key storage; the public behavior control already
passed in strict and non-strict modes. The six new private-API groups could not
run on the parent and are not counted as parent failures.

Four existing test files have explicit startup-accounting migrations. No
resource limit, queue policy, Promise behavior or object identity is changed.
The final suites pass 3,165 tests on Rust 1.99 with `vulkan-raster` and 2,861
on Rust 1.88 with default features, including all 88 confined-worker tests in
each. Strict lint and formatting checks pass. All 102 image-reference pairs
pass, and their 204 PNGs are byte-identical to the preceding Promise.catch
checkpoint.

This checkpoint adds no web API and makes no browser-performance or complete
compatibility claim.

[Evidence](evidence/shared-job-keys/summary.json) retains the pre-implementation
expectations, parent observations, source reviews and validation receipts.
