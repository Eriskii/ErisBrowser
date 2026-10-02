# Exact JavaScript character-data production

JavaScript can now produce exact UTF-16 data, including unpaired surrogates,
through this bounded set of DOM operations:

| Operation | Data behavior |
| --- | --- |
| `new Text(data)` and `new Comment(data)` | Missing or undefined data defaults to empty; other inputs use DOMString conversion |
| `new ProcessingInstruction(target, data)` | Required target; optional data defaults to empty; creation validation still applies |
| `document.createTextNode(data)` | Required data argument, including ordinary conversion of explicit undefined |
| `document.createProcessingInstruction(target, data)` | Both arguments required; creation validation still applies |
| `CharacterData.prototype.data` setter | Raw null becomes empty; undefined or an omitted setter value becomes `"undefined"` |
| `substringData`, `appendData`, `insertData`, `deleteData`, `replaceData` | Read and produce exact units using the complete current splice |

This uses the [storage foundation](dom-strings.md), with one canonical payload:
UTF-8 when the complete result is well-formed, or UTF-16 when it contains an
unpaired unit. There is no second cached representation. A repaired result can
return to scalar storage without changing its code units.

## Conversion and publication

Authentic receiver checks and required argument checks precede conversion.
Methods convert offset, count and data in their specified order, then read the
current node data. `appendData` uses the length after its data callback finishes.
Text and Comment construction converts data before reading the alternate
constructor's prototype. PI construction converts target, then data, then reads
that prototype before validating the target and exact `?>` sequence.

PI creation rejects invalid XML Names and data containing consecutive `?>` with
`InvalidCharacterError`. Unpaired target units remain invalid; data such as
`?\uD800>` does not contain that consecutive pair. Later data assignment and
CharacterData mutation do not repeat creation-only validation, and cloning
preserves mutated PI data and its target.

A mutation classifies its complete prefix, insertion and suffix. An inserted low
unit can pair with a retained high unit; deletion can join a previously split
pair. Unpaired units elsewhere remain exact. Substring can return an isolated
unit and still traverses only its reached prefix and selected result.

All author callbacks finish before fresh storage admission. Creation publishes
its node and optional prototype override only after admission. Replacement
keeps the same node, links and PI target. A refusal does not publish the outer
write; earlier callback effects remain. This is not a transaction that rolls
back author code.

## Work and storage

The shared builder pays for full classification before scanning, then pays for
the chosen output traversal and writes before reserving it. Scalar output goes
directly into one String. Nonscalar output goes into one exact Vec and pays for
its separate canonicality validation. Eliminated borrowed-constructor copies
are no longer performed; other node, arena-growth and prototype-override charges
remain. The explicit builder can change operation cost while keeping the same
fee units and limits.

Replacement admission checks the fresh retained total after subtracting the old
payload, before output allocation, and repeats the check at publication. Script
allocation accounting remains cumulative: an allocated temporary is not refunded
if later work refuses. Detached DOM nodes remain charged. The fixed 100,000-step
author allowance, cumulative estimated 8 MiB script allocation budget, existing
checked per-node bounds, 100,000-node cap and 32 MiB DOM payload cap are unchanged.
The JavaScript substring result still has its 262,144-unit string limit; that
limit is not added to preexisting whole-node mutation input. Existing infallible
allocator paths are not made generally recoverable by this change.

## Reads, worker snapshots and display

Data/length getters, selected text reads, JSON string round trips and cloning
retain exact units. The existing `ERWA` worker protocol carries the canonical
payload unchanged; no wire version or transport cap changes. Layout projects
unmatched units to replacement glyphs without rewriting the node.

The browser fixture creates its retained connected Text through JavaScript.
Load and click check different literal unit arrays on the same node ID. Direct
Page and confined-worker tests inspect exact data and compare its drawing against
an independent scalar replacement-text reference. Native textarea editing still
refuses nonscalar aggregate data rather than feeding a lossy display value back
into an edit.

## Validation and remaining boundaries

The final source passes 1,577 default tests on Rust 1.88 and 1,700 Vulkan-raster
tests on Rust 1.98, with no failed or ignored tests. Strict all-target Clippy
passes for Rust 1.88 with the presenter and Rust 1.98 with Vulkan rasterization.
Focused checks contain 105 distinct tests and 107 successful executions,
including the direct and confined browser witnesses. There are 25 added test
groups compared with the storage foundation. The initial compile correction,
the initially ignored-only worker invocation, and preparation corrections remain
retained observations.

The release adapter passes all 24 modes of the twelve new independent JS groups
and all eight adapter controls; the foundation baseline matched two modes and
all eight controls. The original CharacterData replay advances from 26/34 to
34/34 with its source bytes, success expectations and literal final-unit arrays
unchanged. Across the 4,064 established case modes, these eight gains are the
only changes, and all 364 existing controls retain their complete observations.
PI remains 38/40; its two host-prototype-mutation gaps are unchanged. The two
array-descriptor gains relative to the older pinned gate already existed in the
foundation and are not new gains here. No old report or source is replaced.

The [validation record](VALIDATION.md#exact-javascript-character-data-production)
and [source-bound evidence](evidence/dom-production.json) retain the source,
release fingerprint, full comparisons and initial corrections. The
[foundation CI receipt](evidence/dom-strings-ci.json) records all nine jobs passing.

Legacy `textContent`, `innerText`, title, textarea, append-string and attribute
writes, HTML parser input, scalar form output, exact nonscalar HTML serialization
and unit-aware native editing remain separate work. PI pseudo-attributes,
MutationObserver/Range updates, broader children-changed reactions and complete
Node accessors are still incomplete. This is not full DOMString compatibility,
production security certification or a Chromium performance claim.

Normative references: [DOMString conversion](https://webidl.spec.whatwg.org/#es-DOMString),
[attribute setters](https://webidl.spec.whatwg.org/#es-attributes),
[CharacterData](https://dom.spec.whatwg.org/#interface-characterdata),
[Text](https://dom.spec.whatwg.org/#interface-text),
[Comment](https://dom.spec.whatwg.org/#interface-comment) and
[ProcessingInstruction](https://dom.spec.whatwg.org/#interface-processinginstruction).
