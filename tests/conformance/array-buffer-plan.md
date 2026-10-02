# ArrayBuffer foundation implementation plan

Prepared 2026-09-29 against HEAD `77866e93f4d8486807aa843667619ea4ccccccfa`. This is a plan, with no runtime edits, builds or adapter runs. Implementation remains gated on the independently frozen local/formal oracles and their published-before observations.

## Accepted scope and ownership

Implement custom, non-shared ArrayBuffer construction, fixed/resizable storage, `isView`, the four getters (`byteLength`, `maxByteLength`, `resizable`, `detached`), `resize`, same-realm species-aware `slice`, `transfer`, and `transferToFixedLength`. Install ordinary standard constructor/prototype metadata, `@@species` and `@@toStringTag`.

Only these implementation paths are proposed:

- **new `src/script/array_buffer.rs`**: private state, intrinsic installation, native dispatch, construction, conversions/species, backing operations and accounting.
- **new `src/script/array_buffer/tests.rs`**: semantic and private byte/accounting tests.
- **`src/script.rs`**: module declaration; one Runtime state field/default/bootstrap-size charge; add ArrayBuffer to global/prototype/constructor setup lists; install buffer members after symbols exist; early native dispatch.
- **`src/script/construction.rs`**: constructor allowlist and one specialized construction arm preserving conversion-before-prototype order.

No changes to Value, ScriptObject, parser, numeric-property behavior, generic property algorithms, array helpers, Date, JSON, host transport, quotas or dependencies. Root owns browser/worker fixtures, integration and public docs. SharedArrayBuffer, DataView, typed arrays, externally keyed detachment, foreign realms, Proxy and additional constructor syntax remain separate prerequisites. Existing byte storage has no author-visible index properties.

## Primary references and ordering checkpoint

Current official text was checked on 2026-09-29:

- [ArrayBuffer algorithms and members](https://tc39.es/ecma262/multipage/structured-data.html#sec-arraybuffer-objects), especially [allocation](https://tc39.es/ecma262/multipage/structured-data.html#sec-allocatearraybuffer), [copy/detach](https://tc39.es/ecma262/multipage/structured-data.html#sec-arraybuffercopyanddetach), [resize](https://tc39.es/ecma262/multipage/structured-data.html#sec-arraybuffer.prototype.resize), and [slice](https://tc39.es/ecma262/multipage/structured-data.html#sec-arraybuffer.prototype.slice).
- [ToIndex](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-toindex), [ToClampedIndex](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-toclampedindex), and [SpeciesConstructor](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-speciesconstructor).
- [Byte blocks](https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-createbytedatablock) and [constructor prototype selection](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-getprototypefromconstructor).

Normative ordering to preserve: constructor call rejection precedes coercion; length conversion precedes options lookup; length/max mismatch precedes prototype lookup. Resize validates its resizable brand before conversion, then rechecks detachment and maximum. Slice captures length, converts bounds, constructs and validates species output, rechecks source, then copies only the still-present range. Transfer converts explicit length before checking detachment, allocates from the intrinsic constructor and detaches last. Detached resizable buffers retain their resizable identity while both length getters report zero. This checkpoint supports no views, so isView returns false without author hooks.

## Private representation and invariants

Use an ordinary `Value::Object(id)` and a new `array_buffer::State` stored in Runtime:

```text
State {
    records: Vec<Record>,       // sorted by monotonically increasing object id
    intrinsic: Option<Value>, // saved original ArrayBuffer native constructor
    prototype: Option<usize>, // saved original prototype object id
}
Record {
    object_id: usize,
    bytes: Option<Vec<u8>>,    // Some(empty) is attached; None is detached
    max_byte_length: Option<u64>, // None=fixed; Some persists after detachment
}
```

The detach-key invariant is private `undefined`; no host API can change it. Document that invariant next to detach rather than inventing an externally accessible slot.

Append records only after constructor prototype callbacks and all fallible publication prerequisites finish. Object ids are monotonic; no author callback occurs between object creation and record insertion. Failed ordinary object allocation/publication may retain a charged unreachable object in the existing arena, but must not publish an incomplete branded record. Records are never removed or reordered, including after detachment. A private record index can therefore survive reentrancy, but no Rust borrow or backing pointer can survive any author call.

Implement an explicit, charged numeric binary search over records. Each reached comparison pays before accessing/comparing the id; no text comparisons or linear brand scans. Unknown primitives/arrays/native functions reject cheaply. Saved indices remain stable through append; after every callback reread bytes/length/detachment through the record.

Grow metadata storage geometrically with checked capacities, charged full new capacity and existing-record movement before `try_reserve_exact`. Reserve only when actually full. New arena storage is independent of the existing ordinary object arena; do not claim to fix general arena allocator behavior.

Future views should retain a buffer identity and consult this table on each access, not clone a byte vector or cache a permanent slice. This plan does not install speculative view brands. A dedicated `array_buffer_is_view(&Value)` hook can return false for the present closed set and later consult a separate authentic view table.

## Entry points and reusable operations

Suggested module API:

```text
Runtime::install_array_buffer_intrinsics() -> Result<()>
Runtime::array_buffer_constructor(arguments: &[Value], new_target: Value, doc) -> Result<Value>
Runtime::array_buffer_native(method: &str, receiver: Value, arguments: &[Value], doc) -> Result<Value>
```

Private helpers cover record lookup, ToIndex, clamped slice bounds, ordered allocation, species selection, backing reservation, byte-copy preflight and infallible detach commit. Keep implementation helpers private; only the three integration entry points need `pub(super)`.

Reuse the already charged `splice_number`, `splice_named_get`, and `splice_symbol_get` operations, whose bodies remain untouched. These cover reached property tree comparisons, mapped-name environment lookups, symbol traversal, accessor receiver identity, prototype limits and the explicit unsupported host traversal boundary. Use charged `splice_named_get(new_target, "prototype", doc)` plus the saved prototype fallback in the module instead of silently calling the generic uncharged constructor-property lookup. Species uses general SpeciesConstructor rules, not ArraySpeciesCreate's Array-only/default shortcuts. Undefined constructor and null/undefined species select the saved intrinsic; null constructor is an error.

Use existing `is_constructor` and `construct_with_target` for selected species, with a precharged/fallibly reserved one-element argument vector. Existing bound constructor/newTarget substitution remains authoritative. Ordinary function bodies and generic Construct retain their current budgets and documented broader accounting limitations; do not refactor them here.

Early native dispatch precedes generic argument-content accounting. Methods read only their specified arguments; ignored large extra strings must not be scanned/copied. Constructor calls without `new` reject immediately; methods/getters remain nonconstructable. `isView` ignores receiver and extras and never consults prototype, tag or conversion hooks.

## Concrete operation design

### Construction and installation

Add ArrayBuffer to the three existing global/prototype/native-constructor lists. Its prototype is an ordinary unbranded object inheriting Object.prototype. Existing constructor setup supplies length/name/prototype attributes; length is 1. Initialize state and charge `size_of::<array_buffer::State>()` in Runtime bootstrap. Install after `initialize_symbols`.

Method arities: isView 1, slice 2, resize 1, transfer 0, transferToFixedLength 0; getter and species accessor arities 0. Methods are writable/configurable and nonenumerable. Getter-only accessors are configurable/nonenumerable; their names are `get byteLength`, `get maxByteLength`, `get resizable`, `get detached`, `get [Symbol.species]`. The species getter returns the actual receiver without validation. The prototype tag is `ArrayBuffer`, nonwritable/nonenumerable/configurable. Preserve aliases when globals or prototype properties are replaced/deleted.

Installation must prepay bounded string creation, intrinsic/property-map searches, insertions and metadata storage, using the same B=6 search/structural model as the recent native installers. This bootstrap addition changes real per-runtime retained cost; replay every frozen older profile/control under unchanged caps rather than compensating for it.

ToIndex keeps the full safe-integer domain in u64 until checked conversion for actual allocation: undefined/NaN and either zero yield zero; truncate finite fractions toward zero; reject out-of-range/infinite values with RangeError; Symbol conversion throws. Primitive options are ignored rather than boxed. Only object options read maxByteLength, once; undefined means fixed. Retain conversion effects even when subsequent checks fail.

Ordered allocator: validate length versus optional max; retrieve newTarget prototype; create an ordinary object; stage requested zeroed bytes; perform the late maximum feasibility check; publish its private record. Do not move the maximum check before the prototype getter or initial-block step. Any slot-table reserve necessary for publication is charged before mutation and cannot leave a discoverable branded partial object.

### Accepted feasibility and failure policy

Root accepted the unchanged absolute **MAX_HEAP = 8 MiB** as the maximum-length feasibility ceiling. A resizable maximum above that ceiling is a catchable RangeError at the late AllocateArrayBuffer feasibility step. Storing maxByteLength does not allocate or charge that many backing bytes. Current remaining heap/work refusals remain terminal resource errors. Invalid ToIndex values and length/max mismatch are semantic RangeErrors at their own earlier positions. This is an explicit implementation restriction; retain ordinary upstream high-maximum failures, not inferred exclusions.

An actual backing `try_reserve_exact` failure, after VM preflights allowed the request, maps to the abstract byte-block allocation RangeError. Metadata reserve failure remains a terminal runtime allocation error. Never attempt an oversized reserve to discover the host's physical limit. No new cap or quota increase.

### Resize

Fixed buffers reject before evaluating newLength. A resizable record can be captured by stable index, but its attachment and current length are reread after conversion. If the argument transfers the receiver, detached error wins over maximum range validation. If it recursively resizes, use the resulting bytes and current length.

Implement real in-place shrink with `truncate`, retaining already charged capacity; equal size is a constant-time success. Regrowth within capacity zero-initializes exactly the newly exposed region. Beyond capacity, prepare a new fallibly allocated vector, copy the retained prefix and zero only the extension, then replace backing. Pay all reached work/storage before mutation; inability to grow leaves the current bytes/length intact, apart from prior callback effects. Shrink followed by regrowth must never re-expose discarded bytes.

### Slice

Keep captured length and computed bounds as numbers, not borrowed data. Resolve constructor/species live, then use Construct with exactly the computed requested length. Validate the returned object's actual private brand, attachment, distinct identity and current capacity before the final source recheck.

A custom constructor may return a previously exposed, frozen, nonextensible, resizable or larger buffer. Ordinary property attributes do not prevent writes to internal bytes. The implementation may replace only the copied prefix, leaving all other destination bytes unchanged. Source resizing during conversions/species changes how many bytes remain to copy, not the captured requested result length. A source detached during bounds conversion still permits later specified species effects before the final detached rejection.

After every callback and validation, derive copyCount using checked/saturating bounds from the *current* source. Prepay byte work before any write. Use disjoint record borrows obtained with `split_at_mut`; source and target identity was checked first. No temporary copy vector, no backing clone, no raw pointer and no possible callback while borrows exist.

### Transfer variants

Cache only stable source identity before conversion. Explicit undefined takes current length without conversion; an object argument can detach/resize before the detached check. Ordinary `transfer` preserves the source's resizable maximum; `transferToFixedLength` does not, so it can request more than that old maximum if allocation permits.

Always allocate the destination through the saved intrinsic constructor path, ignoring source constructor/species and mutable global ArrayBuffer. Initial implementation uses a real independently allocated destination and charged byte copy. Zero extension follows constructor allocation; no zero-copy optimization or backing alias yet. Derive copy length from the source after conversion. Prepay copy work and every remaining fallible internal step before writing/copying, then detach source as an infallible final commit. No fallible accounting/lookup/author call may follow detachment. Failed allocation/copy preflight must leave source attached with its current contents; callback effects are preserved. Successful detach drops backing, sets exposed length to zero and retains resizable maximum identity, prototype and ordinary own properties.

## Work and storage model

- Retain MAX_STEPS, MAX_HEAP and call/stack/depth limits verbatim. No refunds for detached/dropped/replaced allocations.
- Byte work is charged per actual linear pass using the existing eight-unit bulk convention: `1 + ceil(bytes/8)` for zero-fill; cover both source read and target write for copies. State this as the VM's logical work model, not cycle or allocator telemetry. No charge for a nonexistent scan of reserved maximum capacity.
- Every new byte vector prepays its entire requested allocation before `try_reserve_exact`; all capacity multiplications, conversion and additions use checked arithmetic. Zero length reserves nothing. Avoid an initial full zeroing pass followed by an unnecessary old-prefix overwrite on resize replacement: copy prefix then zero extension, charging exactly the executed passes.
- For metadata growth pay full new record-buffer storage and relocation work, not only the capacity delta. Live backing operations use checked lengths and ranges; a u64 logical limit never truncates via `as usize` before validation.
- New target ordinary-object metadata uses the existing object allocator and its existing charges. Cached fixed keys, intrinsic names and getter values receive installation storage/work charges. Temporary Construct arguments get their own fallible reservation.
- Work/heap failures before slice copying leave an externally supplied destination unchanged. Transfer detachment and resize publication have no partial-byte resource interruption. Prior author effects remain visible; terminal errors continue to bypass author catch/finally through the existing VM mechanism, without introducing cleanup callbacks.

## Independent private validation seams

Add meaningful tests using real installed intrinsics, not fabricated native identities. Keep JavaScript oracle bytes fixed by the independent owner. Private storage probes may seed and inspect bytes; do not add author-facing mutation APIs.

1. Constructor conversion/option/prototype sequence, early mismatch and late maximum feasibility; null/primitive options; ignored extras; Reflect/bound alternate newTarget; throwing getters.
2. Genuine brand versus forged prototype/tag; unbranded ArrayBuffer.prototype; ordinary own numeric properties; descriptors/arities/nonconstructability; alias persistence.
3. Getter state across zero/fixed/resizable/transfer, with no coercion or backing allocation, plus private lookup growth bound.
4. Resize prefix retention, shrink/regrow zeroing, growth beyond capacity, same-size fast path, nested resize and detach during ToIndex.
5. Slice copy independence; numeric bounds/infinities; exact species/newTarget order and argument; invalid/identical/too-small/detached result before source final recheck.
6. Explicit bytes for source shrink during species: prefill a larger returned target with sentinels and verify only the surviving prefix changes. Source emptied during callback copies nothing. Preserve the untouched suffix rather than assuming zero.
7. Slice callback source growth, source detachment, target ordinary properties/frozen state and source/destination identity. No backing borrow survives any callback.
8. Transfer fixed and preserved-resizable variants; source detachment plus preserved ordinary properties; smaller/equal/larger requested output and zero extension; transferToFixedLength greater than old max; constructor/species poison ignored.
9. One-short work/heap refusal at allocation, metadata publication, growth and copy boundaries, using measured actual prefixes. Verify no detach, no partial backing publication and no modification to preexposed slice target.
10. Repeated small buffers/transfers share cumulative allocation; metadata geometric movement and lookup charges scale with reached work. Discarded storage never refunds quotas.
11. Terminal resource stops suppress catch/finally while preserving earlier conversion/species effects and restoring call/stack counters. Recursive coercion/species use existing depth limits.
12. isView's no-coercion/no-heap path returns false for every currently supported kind, including real buffers; future views are explicitly absent.

Once oracles/before are frozen: implement the four owned paths, run focused new groups and appropriate existing script/construction/integrity/array regressions with immutable failure logs, format and scoped Clippy, then provide source/accounting freeze for independent review. Root performs local/formal candidate replay, full matrices and all older contracts/controls. No result tuning, corpus filtering or baseline change is part of implementation.

## Limits and expected prerequisite effects

ArrayBuffer is not array-like or iterable by default; existing Array.from's ArrayBuffer prerequisite can become an empty-array success. Object sealing/freezing affects ordinary properties and extensibility, not backing slots; a sealed/frozen resizable buffer still resizes, slices and transfers. This requires no exotic property hook.

Existing concat tests eagerly constructing several typed-array classes remain dependent on those missing constructors; this checkpoint does not clear them. Typed-array/DataView/SharedArrayBuffer/Proxy/cross-realm/$262 detachment cases remain visible in complete source inventories. Private byte tests substantiate backing correctness, while public tests cannot yet independently observe arbitrary byte content. General property-map/JSON accounting outside the reused charged operations remains an existing separate scope.

## Read source bindings

The following runtime files were read; hashes are SHA-256. These bindings distinguish this design review from future implementation inputs.
- `src/script.rs`: `450d5aa4d93428085b285fe61fc7fde695f30d95a534a1f2017d2cf5d3765e80`
- `src/script/construction.rs`: `e65151d43abf5e582766e6119c57262bd33e50be2b4ec4de14371515476a8379`
- `src/script/array_splice.rs`: `50f9d4645755f317218c5fb932a273e9d2792d63f2a301d77f95afd25b0f9802`
- `src/script/array_concat.rs`: `fc94a725cb6dcef9e68505ffcbae941ec734d702bf693323868c7eef5ddb7cc1`
- `src/script/iterators.rs`: `09bd3ab4aa3a9e76e6bebde2a91d1c21e23246227027fb9c2d8e18da5388cc34`
- `src/script/symbols.rs`: `35c3ccb263139414afce07f3e785afafe755ecacaf5266e2fee0e0eca44200a9`
- `src/script/object_integrity.rs`: `91e1c99ec7eb5c31cd132b8adea72a7533a55566cbe4676d555227d432b66731`
- `src/script/own_keys.rs`: `622e0f30d5d1a890517ad82a8d75e4bdc8def1f787960f98594fd4e338963f06`
- `src/script/date_builtins.rs`: `595437af6031872894b5f78fcec727bba2e7b7b75496d75696eb7022983847d6`
