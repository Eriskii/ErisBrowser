# Fallible runtime initialization

The browser, JavaScript adapter and stress tool now report checked runtime
initialization failures instead of reaching the constructor's former `expect`
calls. `Runtime::try_new` and `Runtime::try_with_date_host` return `ScriptError`;
an incomplete realm is dropped without being returned to the caller.

Page construction exposes `try_from_html`, `try_from_html_with_date_host` and
`try_error_with_date_host`. Public navigation loaders keep their existing
`Result<Page, String>` API. Internally, the worker distinguishes document errors
from runtime initialization errors, including a failure during encoding reparse.
Only a document error gets an error-page attempt. If either the original realm
or that error page cannot initialize, a writable worker channel receives one
bounded error reply and the worker exits before acknowledging the load.

The adapter still parses the case before capturing its Date host or constructing
a realm. Initialization failure precedes harness and author execution and is
reported as an adapter initialization error, without an author exception identity.
The stress tool similarly reports a harness failure rather than counting the
case as a rejected script.

At the original checkpoint, successful initialization retained its field values, installation order,
Date host assignment, work charges and heap charges. It leaves **70,334 work
units** before any script-entry reset. Numeric quotas and all existing script
and event reset points are unchanged. This change adds no JavaScript or DOM
feature support. A [later DataView startup optimization](evidence/data-view-bootstrap-lookup.json)
removes 21 unused prototype lookups and increases the remaining work to 70,796;
heap charges, object counts and numeric limits stay the same.

The later [work-boundary change](runtime-budget-boundary.md) retains those raw
bootstrap charges, then gives a successfully returned runtime a fresh fixed
author allowance. Heap accounting and the existing script/event entry resets
remain unchanged; the verification below describes the original checkpoint.

`Runtime::new`, `with_date_host`, `Default` and the existing Page convenience
constructors remain documented panicking APIs. The new fallible paths propagate
existing checked work, logical-storage and fallible-reservation errors. They do
not recover from every allocation failure: initial bindings, B-tree/Rc/string
allocations, DOM parsing and other existing paths still contain infallible
allocations. Unrelated panics and allocator aborts are outside this result.

## Verification

Eleven private Runtime groups exercise real quota failures in object reservation,
frame preparation and intrinsic installation, including partial initialization
and exact/one-short late boundaries. They also check fresh-realm metadata
isolation, clock injection and the unchanged direct-helper budget. Five adapter
and worker groups check routing and terminal replies, using typed error injection
for failure branches. These are not physical out-of-memory experiments.

| Check | Rust 1.88 | Rust 1.98 |
| --- | ---: | ---: |
| Default tests, including ignored tests | 1,377 passed | 1,377 passed |
| Native-feature tests, including ignored tests | 1,494 passed | 1,494 passed |
| Strict Clippy, default/presenter/native | 3 passed | 3 passed |

Formatting passes. All sixteen added tests and all prior test identities pass.
The existing exhaustive UTF-16 JSON test is byte-identical and passes in all four
full matrices. All twelve validation commands and the release build passed on
their first attempts.

The release replay preserves every complete case and control record across
44 formal profiles (19,727 cases and 4,564 controls) and 11 local suites
(1,756 cases and 588 controls). All 36 established gates pass with unchanged
baseline bytes. All 48 explicit local resource-policy rows, ordinary failures
and existing exclusions remain unchanged.

The [summary](evidence/runtime-initialization.json) and
[evidence archive index](evidence/runtime-initialization/index.json) bind the
source, release binaries, checks and complete reports. Two inert invocations
omitted the mandatory output-directory argument and stopped before execution.
Data-only audit readers also corrected a feature-specific CLI-test assumption
and a release-record field name. No candidate test, expectation, budget or replay
was changed in response.

The later [Node constant batching checkpoint](node-constants-bootstrap.md)
replaces repeated property insertion with two admitted sorted builds. Its raw
bootstrap leaves 11,022 work units and charges 1,734,936 bytes, with 679 objects,
321 native entries and 25 legacy prototype entries. The fixed work/heap limits
and success-only author grant remain unchanged. Existing blocks stay charged;
eliminating unperformed insertions leaves more logical heap for author work.

The later [Text operations checkpoint](text-operations.md) installs two ordinary
cached function bags for `splitText` and `wholeText`. Raw bootstrap leaves
10,636 work units and charges 1,741,271 bytes, with 681 objects and capacity,
321 native entries and 25 legacy prototype entries.
No work or heap limit and no author-entry reset site changes.

The later [Node normalization checkpoint](node-normalize.md) installs one ordinary
cached method bag and admits the distinct prototype/constructor constant maps.
Measured bootstrap leaves 10,199 work units and charges 1,744,501 bytes, with
682 objects/capacity, 321 native entries and 25 legacy prototype entries. This
adds 437 charged work units and 3,230 bytes. Numeric limits and reset sites stay
unchanged. The method's descendant walk, exact payload copy and removals spend
the existing author allowance; retained effects are not rolled back on refusal.

The later [Node predicate checkpoint](node-predicates.md) installs three ordinary
cached method bags for `hasChildNodes`, `isSameNode` and `contains`. Measured raw
bootstrap leaves 8,807 work units and charges 1,755,715 bytes, with 685 objects
and capacity, 321 native entries and 25 legacy prototype entries. This adds
1,392 charged work units and 11,214 bytes. The larger prototype constant map
needs four allocated tree nodes; the constructor map still needs three.
Numeric limits and author-entry reset sites remain unchanged. Predicate bodies
spend the existing allowance without allocating traversal or payload storage;
generic invocation and inherited diagnostic allocation retain their own limits.

The later [ordinary root checkpoint](node-root.md) installs one cached
`getRootNode` method bag. Measured raw bootstrap leaves 8,318 work units and
charges 1,758,971 bytes, with 686 objects and capacity, 321 native entries and
25 legacy prototype entries. This adds 489 work units and 3,256 charged bytes.
Numeric limits and author-entry reset sites remain unchanged. The root walk
owns no traversal storage; object options separately pay for the fixed member
key and ordinary lookup/callback work before fresh parent links are read.

The later [structural equality checkpoint](node-equality.md) installs one cached
`isEqualNode` method bag. Measured raw bootstrap leaves 7,816 work units and
charges 1,762,227 bytes, with 687 objects and capacity, 321 native entries and
25 legacy prototype entries. This adds 502 work units and 3,256 charged bytes.
The prototype constant batch now guards nine old entries and builds 27 before
the later constructor property; its order buffer reserves 28 slots. Numeric
limits and reset sites remain unchanged. Equality's traversal scratch is charged
separately to author work and cumulative heap; these bootstrap figures do not
measure wall-clock time or physical memory.

The subsequent [connection checkpoint](node-connected.md) adds one cached getter
and expands the prototype constant batch from 27 to 28 entries, with 29 order
slots for the later constructor property. Measured raw initialization retains
7,292 work units and charges 1,765,502 bytes, with 688 objects/capacity, 321 native
registry entries and 25 legacy prototypes. Relative to equality this is 524
additional work units and 3,275 charged bytes, matching the held ledger. The
constructor batch is unchanged; numeric limits and reset boundaries are unchanged.
Three descriptive test literals were updated only after the successful measured
connection test. A separate missed private preconstant count was corrected from
nine to ten after retaining its six failing observations.
