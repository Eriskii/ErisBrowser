# Exact Document.title accessors

`Document.prototype.title` has ordinary configurable, enumerable getter and setter functions. JavaScript title reads preserve UTF-16 units, including unpaired surrogates, and normalize only ASCII whitespace. Writes preserve the raw converted string in a fresh Text child while retaining detached former children and their descendants.

## Selection and conversion

An SVG `svg` document element uses its first direct SVG `title` child. Other getter calls use the first HTML `title` in connected tree order. Ordinary template children participate; the separately associated template content does not. Reads concatenate only direct Text children, ignore nested element text and comments, and strip/collapse tab, LF, form feed, CR and space. Nonbreaking space and vertical tab remain unchanged. Surrogate pairs may span direct Text nodes separated by ignored children.

The setter authenticates its receiver before converting its argument once to DOMString. Explicit null and undefined become their literal strings; Symbol and abrupt conversion throw. Selection happens after conversion callbacks, using the resulting tree and available capacity. A missing document element or a non-HTML, non-SVG-svg root ignores the converted value. An HTML namespace root can update an existing HTML title even when its local name is not html. Creation requires the actual HTML head under an HTML html root. A missing SVG title is prepended to its SVG svg root.

A missing title is created detached, then inserted, then given its contents. Each completed stage remains observable after later resource refusal. Empty input still creates a title but no Text. Nonempty input needs a fresh Text slot even when previous title children are removed. Detached nodes and retained bytes are never credited back. Existing-target replacement uses the same checked container replacement as Node.textContent.

Ordinary lookup supports own shadowing, prototype property replacement/deletion and saved accessor calls. Removing the prototype accessor exposes no old virtual getter or setter. Getter and setter identities are separate, realm-local native functions with length 0 and 1 and names get title and set title. They are not constructors. HTMLElement.title remains its separate attribute route.

## Accounting and presentation

Selection traverses ordinary child edges with an incremental fallible frame vector, bounded by node and depth limits. Name comparisons, reached edges, frame growth and writes are charged. The normalized UTF-16 stream is allocation-free; an exact read plans its output and repeats the same immutable stream into one admitted vector. Runtime separately pays for the final JavaScript string copy. Absolute work and allocation counters are copied back on success and failure.

Missing-title creation admits the static tag, logical node and any actual arena growth before publication. Narrow empty-title insertion validates the actual head or SVG root and pays child-vector growth and prepend shifts before committing links. It cannot move a base element or details group. Later content replacement retains the existing base/details/summary effects and existing B-tree/URL allocation limitations. This is not a process-wide exact allocator or general OOM recovery.

Page metadata projects the same normalized units to Unicode scalars, replacing unmatched units for display. Its valid scalar prefix remains bounded to the existing host byte limit. An empty displayed title uses the page URL. Worker metadata retains its existing 512-scalar cap, which never splits a valid astral character. Exact DOM data still travels in the unchanged ERWA format.

Host title extraction has a separate finite work bound derived from retained DOM bytes, node count and depth. That bound covers the valid document's selection and two projection passes and stops repeated scans in a malformed publicly mutated tree. It does not grant or reset JavaScript work. The getter still uses the caller's remaining author allowance.

## Validation

The final sixteen-path source passes 1,672 default tests on Rust 1.88 and 1,795 Vulkan-feature tests on Rust 1.98, with no failures or ignored tests. Strict all-target Clippy passes for Vulkan rasterization and the Rust 1.88 presenter; formatting passes. Thirty-five test groups were added. Final passing focused commands contain 82 executions of 78 unique tests.

The first compile failed on a test-only ambiguous empty-array assertion; an equivalent is_empty assertion fixed its type. The first bootstrap filter retained 22 passes and two stale descriptive-count failures. Three recorded work/object literals were updated after measurement, without changing quotas or admission outcomes. Strict Clippy later requested a nested-conditional rewrite; a let-chain preserves the same short-circuit, budget and error order. Both full suites were rerun on the final source. Original files, failures, retries and pre-execution depth/formatting corrections are retained.

Raw bootstrap leaves 6,280 work units and charges 1,804,200 bytes, with 679 objects and capacity, 321 native entries and 25 legacy prototypes. Title installation adds 360 work units, including its two accessor bags and subsequent property-insertion costs; no author or initialization allowance increased.

The release preserves all complete rows across 4,138 established case modes and 404 controls. The new 24 ordinary-success title modes advance from 4 to 24, and controls advance from 12 to 16. Those are 20 case gains and four title-dependent control gains, with no losses, other changes or input drift. All independent sources and expectations remain frozen.

The [summary](evidence/document-title.json) and [verified archive](evidence/document-title.tar.gz) bind source snapshots, original inputs, commands/logs, release fingerprint, complete comparisons and reviews. The [preceding append CI receipt](evidence/append-domstrings-ci.json) records nine successful jobs for that earlier published commit; it is separate from title validation.

The independent browser witness loads once and dispatches two actual clicks in the same Page and worker generation. It checks exact raw data, successive Text IDs, detached subtrees, ordinary accessor behavior, normalized metadata and scalar truncation. Six colored squares and a short status string are compared against a complete literal scalar reference canvas. The final long title has 514 UTF-16 units: 511 ASCII characters, one astral pair and one unmatched high unit. Page metadata has 513 scalars; worker metadata has 512 and retains the entire astral character.

This increment does not add XML document parsing, cross-document ownership, mutation observers, custom-element reaction delivery, dirty textarea semantics, complete DOM interfaces or broad web compatibility. Legacy innerText, textarea setters and exact attribute writes remain separate work. No new GPU or Chromium performance measurement is claimed.

Normative references: [HTML document.title](https://html.spec.whatwg.org/multipage/dom.html#document.title), [DOM child text content](https://dom.spec.whatwg.org/#concept-child-text-content), [ASCII whitespace normalization](https://infra.spec.whatwg.org/#strip-and-collapse-ascii-whitespace) and [Web IDL attributes](https://webidl.spec.whatwg.org/#es-attributes).
