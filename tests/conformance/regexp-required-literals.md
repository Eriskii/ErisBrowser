# Required literals before regexp backtracking

Patterns with variable repetition can repeatedly explore input that lacks a
necessary delimiter. The custom compiler now derives conservative lower bounds
for at most four literal UTF-16 units. Before matching, a bounded scan rejects
input with too few occurrences, avoiding construction of backtracking states.

Sequences add counts; alternatives retain shared requirements with the minimum
count; positive repeat minima multiply with saturation. Optional repeats,
classes, assertions and backreferences add no requirements. Assertion text can
overlap consuming text, so counting it would be incorrect. Dropped candidates
only weaken the check. The scan uses the VM's existing non-Unicode case folding,
starts at the requested input index and stops once all retained counts are met.
Matching order, captures and global/sticky index handling remain with the VM.

Analysis is iterative over the bounded node arena. Its temporary fixed-size
records are charged before allocation; only the root's record survives. The
scan uses fixed storage and charges its input visits and comparisons. Patterns
without both variable repetition and literal nodes skip this analysis. No
pattern, node, repetition, work, heap or native-stack quota changes.

## Frozen comparisons

Both unchanged modes of Test262 `S15.10.2_A1_T1.js` now complete all 25 XML
patterns within the shared script budget. The complete 488-source constructor
directory reaches **774 passed, 200 unsupported and two resource stops**. Those
remaining stops come from the full UTF-16 script loop. Every other observation
in the **28 profiles / 9,078 modes** is identical to the preserved `45fb59e`
release, as are all **2,288 assertion controls**, source fingerprints, policies
and 23 healthy baseline gates. The constructor inventory remains an observation
profile; its remaining resource stops are not hidden or reclassified.

The [local fixture](regexp-required-literals.js) was frozen before implementation.
Its **eight resource stops and 22 passes become 30 passes** without source or
mode-fingerprint changes. It checks absent and present delimiters, alternatives,
optional paths, assertions, references, case folding, surrogate/NUL units and
global/sticky indices. Rust tests additionally cover precharged allocation
failure, scan exhaustion, saturating count arithmetic, capture equality against
the unfiltered VM and rejection without VM allocation.

A direct preserved/current comparison retains **3,888 identical parse outcomes
and 106,040 identical full capture/error outcomes**. These preservation checks
do not establish independent conformance.

## Timing scope and validation

Seven alternating local samples per synthetic workload measure direct Rust
compile/find calls. Relative to the preserved implementation, measured ordinary
positive matching medians range from effectively unchanged to about **6% higher**.
Compilation medians increase about **20% for `a+b`** and **13% for the large XML
pattern**. Missing-prefix matching improves in this sample. The old missing-
delimiter case exhausts its budget while the new implementation returns no match;
its timing ratio is therefore not a completed-work speedup. These are not browser
workloads or a Chromium comparison. All raw samples and the harness source are
retained in the [machine-readable evidence](regexp-required-literals.json).

Rust 1.88 and 1.95 pass strict all-target Clippy and **1,023 default / 1,034
Vulkan-feature tests**, none ignored. Both release builds preserve **57 CPU pixel
references**. **174 Python tests**, **45,000 deterministic mutation cases** and
unchanged HTML observations pass. No independent agent review or GPU exercise
was performed. Unicode regexp syntax, broader grammar/protocol coverage, general
matching performance, full web compatibility and audited security remain open.
