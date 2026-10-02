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
does not repeat the constructor's `?>` restriction. Clone, tree insertion,
serialization and worker snapshots retain the stored target and data.

Checked storage takes ownership of converted UTF-8 strings. Admission failure
refuses the operation instead of truncating data. Replacement accounts for the
old buffer's released bytes and preserves node identity, tree links and PI
target. Node publication and alternate-prototype publication follow all checked
admission. Existing work, heap, document byte and node limits remain unchanged.

This is partial CharacterData/ProcessingInstruction support. Lone UTF-16
surrogates in data remain explicitly unsupported because document strings use
UTF-8; lone surrogates in a target are invalid XML names. PI pseudo-attribute
methods and their update hooks, mutation observers, range maintenance, remaining
CharacterData methods, complete Node attribute descriptors and host prototype
mutation are unfinished. Independent Document construction remains unsupported.

The independent local fixture checks 20 cases in both script modes: 38 of 40
expectations pass. Both modes of the original host-prototype-mutation case remain
unmet; the separately authored constructor-override brand case passes. These are
local behavior tests, not a complete upstream DOM conformance suite. See the
[validation record](VALIDATION.md#processing-instruction-and-characterdata-accessors)
and [retained evidence](evidence/processing-instruction.json).

Normative references: [ProcessingInstruction](https://dom.spec.whatwg.org/#interface-processinginstruction),
[Document factory](https://dom.spec.whatwg.org/#dom-document-createprocessinginstruction),
[CharacterData](https://dom.spec.whatwg.org/#interface-characterdata),
[Web IDL attributes](https://webidl.spec.whatwg.org/#es-attributes) and
[XML names](https://www.w3.org/TR/2008/REC-xml-20081126/#NT-NameStartChar).
