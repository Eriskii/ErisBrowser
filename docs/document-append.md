# Checked DOM append

Document now exposes its own `append` function alongside Element and
DocumentFragment. Saved methods check the receiver before converting arguments;
function identity, name and length remain specific to the defining interface.

All variadic arguments convert in order before insertion begins. String
arguments become text nodes before any argument node moves. Multiple arguments
assemble in a temporary fragment in their original order, including duplicates,
then undergo the destination's live hierarchy checks. A later error preserves
earlier conversion effects and node moves, including changes to the document's
base URL and named disclosure groups.

For example, `document.append(document.documentElement)` throws
`HierarchyRequestError`: the document already has an element. Passing that root
twice first moves it into a temporary fragment, then restores it as the single
document element. Appending the root and a text string removes the root during
assembly but fails final insertion, leaving the root in the temporary fragment.

The script-facing `appendChild` path shares these checks. Invalid parent kinds,
host-including cycles, misplaced doctypes, text under a Document and multiple
document elements throw DOMException with name `HierarchyRequestError` and legacy
code 3. Implementation work, storage and depth limits remain terminal resource
errors. Text admission checks the actual UTF-8 storage before calling the DOM
builder; this path cannot silently truncate an admitted string.

Nineteen append test groups cover conversion, mutation prefixes, node kinds,
document child ordering, base URLs, disclosure groups, exact/one-short resource
boundaries and terminal cleanup. Two storage groups check the shared DOM ledger
and multibyte text admission. A six-case HTML fixture runs through both direct
Page and confined-worker paths, checking literal pixels and result text on load
and click.

Rust 1.88 passes 1,401 default tests and Rust 1.98 passes 1,518 tests with native
Vulkan rendering enabled, including ignored confinement tests. Strict native
Clippy and formatting pass. The unchanged DOM identity corpus gains two passing
modes, reaching 54/58 verified expectations with all 44 controls healthy. The
Object.is local corpus retains every complete case and control record (72/78
expectations, 60 controls). The [raw results](evidence/document-append.json)
retain both before and after observations, including remaining failures.

Real interface prototypes, ordinary host-member replacement, live NodeList
semantics, mutation observers, broader adoption and exact lone-surrogate DOM
storage remain unfinished. The parser's raw insertion routine is unchanged.

Normative references: [DOM append](https://dom.spec.whatwg.org/#dom-parentnode-append),
[pre-insertion validity](https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity)
and [Web IDL variadic operations](https://webidl.spec.whatwg.org/#es-operations).
