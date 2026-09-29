# Array indexed and length descriptor fixtures

This checkpoint freezes **36 independently authored cases / 72 strict and
sloppy variants** before running the implementation candidate. Each `// CASE:`
block in [array-descriptors.js](array-descriptors.js) executes in a fresh runtime
with unchanged pinned Test262 `assert.js`, `sta.js` and `propertyHelper.js`.
Every case expects normal completion after its assertions. Unsupported features,
resource stops, timeouts and other errors remain visible failures to meet that
expectation; they are not converted into passes.

The saved published release at commit
`48fce52dc414e75d69af3e750227faf48c71ec3f` records **8 passed / 62 unsupported /
2 resource stops**. Its adapter SHA-256 is
`bbafaea885b1f989087877715584f1680f26c1c8fff5f6cf73cf52afc69bf767`.
The two resource stops occur while constructing the maximum logical-length
array. The source SHA-256 is
`32537a09fbc2f87052e183e2467cdbf4f1128bc2b2a7ec817c776ef61c44400d`.
[Machine-readable evidence](array-descriptors.json) retains every source,
mode, fingerprint, expected observation, harness hash and before observation.
The release candidate passes **all 72 unchanged modes**. All per-mode source,
harness and expectation fingerprints match the frozen before record. The
[complete pinned upstream inventory](test262-array-descriptors.md) separately
covers 1,793 sources / 3,574 modes.

Coverage includes:

- Length conversion twice, fresh method lookup, conversion mismatch and abrupt
  effects, mutation before the old length is read, and descriptor-field order.
- Descending partial shrink failure, deferred readonly state, and deletion that
  never calls indexed getters.
- Sparse indices at `2^32 - 2`, ordinary names at `2^32 - 1`, noncanonical names,
  symbols, holes, and inherited properties.
- Data/accessor transitions, exact descriptor flags, SameValue, failed deletion,
  readonly length and nonextensible arrays.
- JSON reviver replacement through data definition, failed replacement/deletion,
  nonextensible holes and saved iteration length after a readonly shrink.
- Existing push/pop/reverse/sort/map and JSON serialization consuming live array
  properties, including partial copy-back effects.

The expectations follow the current [Array exotic object algorithms](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-array-exotic-objects)
and [ArraySetLength](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-arraysetlength).
Reviver cases follow [InternalizeJSONProperty](https://tc39.es/ecma262/multipage/structured-data.html#sec-internalizejsonproperty),
including its nonthrowing treatment of rejected data definitions and deletions.

Execution uses the existing Test262 adapter runner with a three-second limit
per mode, bounded captured output, inherited 512 MiB address-space and
30-second process CPU limits, and a 45-second outer runner deadline. Large
logical lengths have only a few occupied properties; the fixtures do not
require dense storage or a full sparse-range scan. This focused inventory is
not a complete Array conformance suite, allocation audit, security assessment,
GPU exercise or Chromium performance comparison.


Implementation stores logical length independently of allocated dense elements
and sparse descriptor records. Growth allocates no gaps. Shrink visits actual
own indices in descending order, restores length after a blocking element and
applies a requested readonly flag even after partial deletion. Length values
undergo two observable conversions before current length/writability are read.
`Object.defineProperty`, `Object.defineProperties`, `Reflect.defineProperty`,
ordinary writes, deletion and JSON revival share these rules. RegExp's fresh
result arrays retain private data creation rather than inherited assignment.

Push, pop, shift, unshift, join, indexOf, includes, slice, forEach, map and filter
now use live property operations on supported ordinary array-like receivers.
They preserve holes/inherited entries, callback receivers, partial write effects
and string work charges. Join suppresses actual cycles and clears its recursion
marker after failures. Slice/map/filter support default/null/undefined species;
custom species constructors remain explicitly unsupported.

JSON revival/stringification still preallocate a charged key snapshot from
logical length; very large sparse arrays can stop before their first indexed
callback. Sort/reverse retain their explicit 65,536 logical-length limits.
Sparse scans and all callbacks still use unchanged work/heap/recursion budgets.
No quota was increased. Tests that formerly used missing array descriptors or
large sparse lengths as error fixtures now check the supported behavior; actual
dense growth and persistent page allocation still test resource termination.

Validation: Rust 1.88 and 1.95 each pass strict all-target Clippy and 1,053
default / 1,064 Vulkan-feature tests. The 189 Python tests, 15,000 mutation cases,
unchanged HTML observations and 57 pixel references in both release configurations
pass. Across 32 upstream profiles, 13,437 modes and 2,612 assertion controls are
retained. The new descriptor profile becomes the 26th healthy regression gate.
