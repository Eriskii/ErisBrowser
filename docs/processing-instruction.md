# Processing instructions and character data

`new ProcessingInstruction(target, data)` and
`document.createProcessingInstruction(target, data)` create genuine nodes in the
active document. Targets follow XML 1.0 Fifth Edition `Name`, including colons
and its specified Unicode ranges. Creation rejects invalid targets and data
containing `?>` with `InvalidCharacterError`.

The constructor requires a target; omitted or undefined data defaults to `""`.
The Document factory requires both arguments and converts explicit undefined
data to `"undefined"`. Argument conversion precedes validation. Construction
also reads `newTarget.prototype` after conversion and before validation. Author
exceptions propagate with their original identity, and document capacity is
checked again after callbacks finish.

`ProcessingInstruction.prototype.target` is a read-only accessor.
`CharacterData.prototype.data` is a mutable accessor shared by represented Text,
Comment and ProcessingInstruction nodes; `length` counts UTF-16 code units.
These are ordinary configurable, enumerable prototype properties. Saved
accessors check authentic node state independently of prototype membership.
Replacing, deleting or shadowing a property uses ordinary property behavior.

The data setter checks its receiver before conversion. Its Web IDL
`LegacyNullToEmptyString` conversion maps null to `""`; undefined, including a
saved setter called with no arguments, becomes `"undefined"`. Setting PI data
does not repeat the constructor's `?>` restriction. Clone, tree insertion and
worker snapshots retain the stored target and data. Exact reads and explicit
serialization/presentation boundaries are described in [DOM strings](dom-strings.md).

[Exact JavaScript data production](dom-production.md) now preserves unpaired
UTF-16 units through PI creation and the CharacterData data setter. Complete data
chooses one scalar UTF-8 payload or one exact UTF-16 payload, with two retained
bytes per unit in the latter. PI target validation remains separate. Admission
failure refuses the operation instead of truncating data. Replacement accounts for the
old buffer's released bytes and preserves node identity, tree links and PI
target. Node publication and alternate-prototype publication follow all checked
admission. Existing work, heap, document byte and node limits remain unchanged.

This remains partial CharacterData/ProcessingInstruction support. Exact data
production does not make unmatched units valid in PI targets: those remain
invalid XML names. The [storage foundation](dom-strings.md) keeps data exact in
reads, clones and tagged snapshots. PI pseudo-attribute methods and their update hooks, mutation observers,
range maintenance,
complete Node attribute descriptors and host prototype
mutation are unfinished. Independent Document construction remains unsupported.
The [CharacterData follow-up](character-data.md) adds the five substring and
mutation methods with exact UTF-16 substring results and checked canonical
scalar or exact-unit storage. The producer release keeps the original PI replay
at 38/40; its two host-prototype-mutation expectations remain unmet.

The original PI checkpoint checked 20 local cases in both script modes: 38 of
40 expectations passed. Both modes of the original host-prototype-mutation case
remained unmet; the separately authored constructor-override brand case passed.
These historical observations are not a new foundation replay claim. They are
local behavior tests, not a complete upstream DOM conformance suite. See the
[validation record](VALIDATION.md#processing-instruction-and-characterdata-accessors)
and [retained evidence](evidence/processing-instruction.json).

Normative references: [ProcessingInstruction](https://dom.spec.whatwg.org/#interface-processinginstruction),
[Document factory](https://dom.spec.whatwg.org/#dom-document-createprocessinginstruction),
[CharacterData](https://dom.spec.whatwg.org/#interface-characterdata),
[Web IDL attributes](https://webidl.spec.whatwg.org/#es-attributes) and
[XML names](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-NameStartChar).
