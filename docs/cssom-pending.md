# Persistent pending inline declarations

For `padding: var(--p) !important`, setting `padding-left` to a normal value
now replaces just the left side. Removing one side deletes it even though its
public value is empty. The surviving sides retain their variable source and
priority, so a later change to `--p` affects them. Unrelated style edits,
separate script entries, resizing and worker rendering preserve that state.

Each supported shorthand expands to ordered constituent longhand records.
Pending records share an immutable original shorthand and its token string;
ordinary longhands and custom properties retain ordinary tokens. Replacement
keeps an existing record's position, while removal followed by insertion
appends it. `length` and indexed reads count individual retained longhands.
Declaration parsing still honors priority when choosing duplicate winners;
`setProperty` replaces the selected property's priority.

The [CSS Values pending-substitution rules](https://drafts.csswg.org/css-values-5/#pending-substitution-value)
require empty observable pending longhands and reconstruction of a complete
original shorthand. A partial group therefore serializes empty longhand
values. For example, replacing the left side of an important pending margin
with `9px` gives `margin-top:  !important; margin-right:  !important;
margin-bottom:  !important; margin-left: 9px;`. Source content, rather than
shared-pointer identity, determines whether a complete shorthand can be rebuilt.

Serialized text cannot preserve that partial group's unobservable source.
The [CSSOM attribute-change rules](https://drafts.csswg.org/cssom/#css-declaration-block)
therefore matter: external replacement reparses the text, including equal-text
replacement. Assigning `style.cssText` to itself has the same consequence.
The empty pending declarations disappear; explicit longhands and custom values
remain. An ordinary CSSOM edit instead commits its text and typed state together.
These draft specifications were checked on 2026-10-11; the tests are authored
locally, not imported WPT results.

A private sparse document map owns immutable declaration blocks. Script finishes
author conversions before reading the latest state. An edit stages a checked
clone, mutation and serialization, then validates DOM retention and allocation
allowances before publishing either representation. Every checked failure
preserves both representations as they stood after author conversions; completed
nested author mutations remain visible. Successful external style writes release
the old entry; legacy external prefix truncation remains unchanged. Detaching or
reparenting a node preserves its state. DOM `cloneNode` copies attributes with no typed entry; later style access
reparses them. A semantic Rust `Document::clone` shares the immutable block.

The worker's authoritative Page retains typed state. Its UI document snapshot
explicitly projects attributes and skips the private map entirely; it subtracts
a cached retained total without copying and clearing the map. The existing
wire codec remains attribute-only. Such a snapshot is not a semantic round trip
of partially pending declarations. Cascade borrows the retained records and
uses existing late variable resolution and the consuming document's URL base.
A failed typed lookup or traversal never retries through lossy attribute text.

The existing limits remain: 256 expanded records, 64 KiB each for input,
logical declaration storage and serialized output, 4,096 bytes per value and
component depth 16. Every pending reference pays its full token payload in
logical storage even when shared. Typed entries also include vector/string
capacities, Arc storage and a conservative tree envelope in the existing
32 MiB retained DOM allowance. Cached summaries fund editing clones, release
work and snapshot projection. Allocating queries and edits reserve scratch from
logical storage, not the potentially short attribute string; metadata-only
borrowed reads avoid a full parser scratch reservation.

All lookup, expansion, traversal, serialization and commit work uses existing
caller budgets. Checked clone and prepared-value charges also fund cleanup if
a later step refuses the operation. Work and cumulative allocation charges are
not refunded. These are logical admission policies; infallible allocator aborts,
total process memory and native stack usage remain separate concerns. Direct
Rust writes through public raw node/attribute fields can bypass the checked API.

Ordinary serialization retains the existing expanded-longhand format. Complete
preferred-shorthand selection, broader value grammars, stylesheet/rule APIs,
computed style, full prototypes/reflection, namespace APIs and typed IPC transport
remain incomplete. This increment adds no rendering command, shader, resource
fetch authority or broader CSS property inventory.

Run `./run.sh ./examples/cssom-pending.html` for a comparison of a whole
shorthand, a replaced side and a removed side. The buttons change the variable,
reparse one box or restore the initial comparison.

Validation used 739 source inputs. Rust 1.88 passed all 3,046 tests with
`vulkan-raster` and 2,742 with default features, including ignored tests.
Strict all-target Clippy passed on Rust 1.88 and 1.98; formatting passed.
Forty-eight new Rust groups cover records, storage, cascade/bindings, Page pixels
and the confined worker. The parent passed the initial pixel control but failed
all five Page mutation cases and the worker's first edit before implementation.

All 102 self-authored reference pairs passed; all 204 PNGs are byte-identical
to the parent's published archive. The four demo states rendered without
diagnostics, and visual review checked the changed and reparsed layouts after
a label-wrap correction. Native pixel witnesses interpret packed plans on the
CPU. These checks establish no GPU, acquired-window or Chromium-relative result.

Rust 1.88 x86-64 DWARF reports `Document` growing from 400 to 432 bytes and
`InlineStyle` from 24 to 72 bytes. `Node` (136), `Element` (96), `ComputedStyle`
(1,032), `PaintOwner` (16), `BoxGeometry` (72), `Fragment` (136), `InlineItem`
(176) and `InlineKind` (152) are unchanged. These are type sizes, not total
allocator usage or stack-frame measurements.

The first strict check rejected a reference-to-Cow helper parameter. Passing
that value into and back out of the helper preserves variant-sensitive scratch
charges without a clone or lint suppression; both complete suites passed again.
An earlier worker baseline selected zero tests because of a module-path error;
that attempt is retained and excluded from pass claims. The corrected baseline
selected the intended test and failed its first edit. The DWARF reader initially
conflated Document with an enum variant of the same name; qualified enclosing
type names corrected that measurement helper before reported sizes were written.

Final source inventory SHA-256 is
`84990a647a889f090d95df0506ce8c6abf5e6a73c29aba1735c2045bd5d9ac00`.
The frozen browser SHA-256 is
`83c4f7a3658e142cf63310b63aa2edbafb5d41baebfcea9f0d0a647d6824e0b5`.
The complete suites, strict checks and browser build used inventory 2; inventory
3 changes only two demo text labels. Every compiled source and test is unchanged.
The parent `1bbb31ef` passed all nine [GitHub checks](https://github.com/Eriskii/ErisBrowser/actions/runs/38159056564);
this increment's CI runs after publication.

The [evidence summary](evidence/cssom-pending/summary.json) and
[source/check archive](evidence/cssom-pending/sources-and-checks.tar.gz) retain
source bindings, literal expectations, raw results, corrections and reviews.
They do not establish full CSSOM conformance or production security.
