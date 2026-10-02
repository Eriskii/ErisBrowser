# CharacterData methods

Represented Text, Comment and ProcessingInstruction nodes share five ordinary
methods on `CharacterData.prototype`:

| Method | Required arguments | Result |
| --- | --- | --- |
| `substringData(offset, count)` | 2 | The selected UTF-16 code units |
| `appendData(data)` | 1 | Append text; return undefined |
| `insertData(offset, data)` | 2 | Insert text; return undefined |
| `deleteData(offset, count)` | 2 | Remove code units; return undefined |
| `replaceData(offset, count, data)` | 3 | Replace code units; return undefined |

Offsets and counts use Web IDL unsigned-long conversion: Number conversion,
truncation and wrapping modulo 2³². NaN and infinities become zero. The receiver
and required argument count are checked before conversion. Arguments convert
left to right, and the operation reads the current node data after all callbacks
finish. An offset beyond that current length throws `IndexSizeError` with legacy
code 1. Counts clip to the available suffix.

Method data arguments use ordinary DOMString conversion: null becomes `"null"`
and explicit undefined becomes `"undefined"`. This differs from the existing
`data` setter's null-to-empty conversion. Missing required arguments throw.
Saved methods use authentic node state; prototype membership alone does not
satisfy their receiver checks. Replacing, deleting or shadowing a method follows
ordinary property behavior.

Substring results preserve individual UTF-16 units, including half of a
surrogate pair. Only the selected result is subject to the JavaScript string
length limit; traversal is bounded by the unchanged work allowance. Mutations
validate the complete splice before publishing scalar output. A retained
unit and an inserted unit may repair a pair. These runtime methods still require
scalar data in the target node and refuse a final unpaired sequence, with the outer write
refused. This is an implementation gap, not a DOM exception required by the
standard.

The [DOM string foundation](dom-strings.md) now stores host-supplied unpaired
units canonically and preserves them through data/length getters, selected text
reads, cloning and ERWA snapshots. It does not yet migrate these five methods or
the JavaScript constructors/setters to produce exact nonscalar data. The eight
existing standards-success modes remain unmet pending their next replay; their
sources and historical refusal observations remain unchanged.

Replacement preserves the node's identity, links and PI target. Checked storage
accounts for released old data when checking the document byte limit, while the
runtime allocation ledger remains cumulative. A later refusal preserves earlier
author callback effects. No work, heap or DOM limit is raised.

Page rendering collects current text and styles on each layout. The broader
children-changed reactions remain incomplete, including MutationObservers,
live Ranges, textarea dirty/default value separation and PI pseudo-attribute
updates. CDATASection, `splitText`, `wholeText`, normalization and complete Node
attribute descriptors also remain unfinished.

The original CharacterData release replay passed 26 ordinary cases and retained
eight unmet standards expectations for final lone-surrogate DOM data. Separate
checks verified terminal refusal and unchanged outer storage for those eight
modes. PI and selected compatibility results were unchanged at that checkpoint.
These are historical results, not a new foundation replay claim. See the
[validation](VALIDATION.md#five-characterdata-methods) and
[evidence](evidence/character-data.json) records.

Normative references: [CharacterData](https://dom.spec.whatwg.org/#interface-characterdata),
[substring data](https://dom.spec.whatwg.org/#concept-cd-substring),
[replace data](https://dom.spec.whatwg.org/#concept-cd-replace),
[Web IDL operations](https://webidl.spec.whatwg.org/#es-operations) and
[unsigned long conversion](https://webidl.spec.whatwg.org/#es-unsigned-long).
