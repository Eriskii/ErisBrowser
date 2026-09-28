# Metered declaration-name validation

Scope, for-header, function-parameter and catch-binding validation now use bounded
borrowed name records. Growing a record vector charges cumulative storage and
relocation work before a checked reservation. An iterative stable merge sort
charges its scratch allocation, record copies and each string comparison before
performing them. Names and syntax remain borrowed; the helper copies neither.
The record cap is the existing 32,768-token ceiling. No quota changes.

After sorting, adjacent comparisons select the earliest repeated source occurrence,
including whether its diagnostic names a lexical or block-function binding. A
merged intersection preserves the old first lexicographic var conflict. Fallible
binary searches retain for-header and parameter/body checking order. Named error
messages charge their allocation and copy work before construction. Scopes without
direct lexical declarations retain the existing skip of unnecessary analysis.

The [comparison record](scope-names.json) preserves **144 strict/sloppy modes**
from **72 distinct root-authored sources**. Both the frozen `2715048` adapter and
the final adapter match every expected parse status and exact diagnostic. The
[fixture](../fixtures/scope-names.tsv) covers duplicate/source ordering, sorted
intersections, function boundaries, Unicode/escaped/shared-prefix names, loop
headers, catch bindings, strict and sloppy parameter rules, defaults/rest/arrows,
methods and shadowing. It also retains the existing 1,000-long-parameter compile
shape. These local expectations are not upstream conformance evidence.

Five private test groups compare deterministic generated lists with independent
standard-library sorting/set oracles, check borrowed pointer identity, and exercise
work, growth, scratch, comparison, diagnostic and count-limit refusal boundaries.
Empty and singleton lists need no sorting scratch. Allocation accounting remains
cumulative, including temporary buffers that have subsequently been released.

Every case and preflight fingerprint, policy and observation remains identical
across **20 Test262 profiles / 6,879 modes / 1,680 controls**. All controls verify;
fifteen healthy baseline gates pass. The five resource-stopped profiles remain
nonpassing observations and have no new baselines. HTML remains at 3,868 matches,
two mismatches and six unsupported modes.

Final validation passes **917 Rust tests**, **151 Python checks**, **57 exact pixel
references** and **15,000 deterministic mutation cases**, plus formatting, strict
all-target Clippy and release compilation. No independent-agent review,
native-window, Vulkan or Chromium performance measurement is claimed.

The algorithm collects and sorts a whole lexical list before selecting a duplicate;
on sufficiently large inputs, resource exhaustion can therefore precede a syntax
error that the previous insertion-based implementation reported earlier. Duplicate
names consume temporary records. Other parser operations and runtime maps need
separate accounting; owned syntax and parser/evaluator execution retain recursive
paths. This change does not establish full compatibility or production security.
Flat code ownership and explicit continuations remain on the
[roadmap](../../docs/ROADMAP.md#javascript-execution-depth).

Run the focused checks with:

```sh
cargo test --locked --lib script::names::tests
cargo test --locked --test scope_names
```
