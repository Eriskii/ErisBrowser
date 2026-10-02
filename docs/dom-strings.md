# Exact DOM string storage

Text, Comment and ProcessingInstruction data now use an opaque `DomString`
with one canonical payload: `Scalar(String)` for well-formed Unicode stored as
UTF-8, or `Units(Vec<u16>)` when an unpaired surrogate occurs. There is no cached
second representation or implicit `Display`/`Deref<str>` conversion. PI targets,
element names and attributes keep their existing scalar storage.

This is a storage and consumer foundation. JavaScript constructors, setters and
mutation methods have not yet migrated to exact nonscalar production. Older
legacy writes can still replace unmatched units; newer CharacterData writes
still refuse them. The eight existing [CharacterData](character-data.md)
standards-success modes remain unmet in the final release replay. Their original sources and
historical refusal records are preserved. This change does not establish full
DOMString compatibility, production security or Chromium performance parity.

## Ownership and bounds

Retained character data costs its UTF-8 byte length for Scalar or two bytes per
unit for Units. Checked owned creation and replacement move the admitted
payload. Replacement accounts for the old payload before checking the shared
limit; detached nodes remain charged. Snapshot admission recomputes this ledger.
The 32 MiB document payload limit, existing node limits and script budgets are
unchanged. Snapshot decoding does not introduce an 8 MiB per-node restriction:
the existing parser-expanded 18 MiB scalar snapshot remains admissible.

On the checked 64-bit target with Rust 1.88, the payload header grows from the 24-byte `String`
to the 32-byte `DomString`. The enclosing `NodeKind` remains 96 bytes and `Node`
remains 136 bytes; the enum's other variants already determine those sizes.
This observation is not an ABI guarantee across targets or compiler versions.

Exact script reads charge traversal, selected units and output copies before
allocation. Empty and single-Text containers share the CharacterData leaf fast
path and avoid arena-sized scratch for unrelated nodes. Larger trees retain
bounded traversal and scratch admission. Checked reservations do not turn every
existing allocation in the browser into a recoverable operation.

## Worker transport

The worker protocol uses the four-byte version `ERWA`, replacing `ERW9`. Only
Text, Comment and PI data gain a tagged payload:

| Tag | Count | Payload |
| --- | --- | --- |
| 0: Scalar | UTF-8 byte count | Valid UTF-8 bytes |
| 1: Units | UTF-16 unit count | Exact little-endian 16-bit units |

The decoder checks tag, checked byte size, remaining frame and shared DOM budget
before reserving payload storage. Units must contain an unpaired surrogate;
an empty or fully scalar Units encoding is noncanonical and rejected. The
remaining fields, including PI targets, retain their scalar encodings. Old
protocol versions are rejected. The existing 128 MiB response-frame cap and
32 MiB DOM payload cap remain unchanged.

Graph, namespace, detached-node, template-ownership and other snapshot checks
continue to apply. The independent `EWB1` raster capture format contains drawing
data rather than DOM strings and is unchanged.

## Exact reads and explicit boundaries

Data and length accessors, selected existing text readers and cloning preserve
exact data supplied by the host or worker transport. Descendant text units are
joined before decoding, so a surrogate pair can cross adjacent Text nodes.
`textarea.value` reads preserve units while normalizing CRLF and CR to LF. These
paths do not add complete Node attribute descriptors or children-changed
reactions.

Presentation explicitly replaces unmatched units without changing stored data.
CSS direct-child text and SVG text aggregate before replacement. HTML used for
inline SVG carries a pending high unit across adjacent serialized Text pieces;
actual markup separates those pieces. Layout still handles individual text
runs, so this is not complete cross-node Unicode shaping.

The inline JavaScript loader requires a scalar aggregate and reports unsupported
nonscalar source before execution. JavaScript HTML serialization likewise
refuses a final unmatched sequence pending an exact-unit serializer. Its bounded
streaming path now reports resource failure for oversized output instead of
silently omitting later child pieces. Existing serialization coverage gaps
remain separate.

The native editor still owns a UTF-8 buffer. A textarea whose aggregate is
nonscalar cannot enter or retain that editor, including while an edit
acknowledgement is pending. This prevents whole-string edits from replacing
preserved units elsewhere in the value. Unit-aware native editing is unfinished.
Form output and native titles remain scalar presentation/encoding boundaries,
distinct from JavaScript DOMString values.

The final source passes 1,552 default tests on Rust 1.88 and 1,675 Vulkan-feature
tests on Rust 1.98, including confinement checks. All 4,064 compared case modes
and 364 controls retain their observations and expectations. See the
[validation record](VALIDATION.md#exact-dom-string-storage) and
[source-bound evidence](evidence/dom-strings.json), including initial failures
and corrections. Earlier [PI](processing-instruction.md) and
[CharacterData](character-data.md) records describe their original checkpoints.

Normative references: [DOMString conversion](https://webidl.spec.whatwg.org/#es-DOMString),
[CharacterData](https://dom.spec.whatwg.org/#interface-characterdata),
[HTML fragment serialization](https://html.spec.whatwg.org/multipage/parsing.html#serialising-html-fragments)
and [CSS input preprocessing](https://drafts.csswg.org/css-syntax-3/#input-preprocessing).
