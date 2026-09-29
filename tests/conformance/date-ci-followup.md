# Date Rust 1.98 Clippy follow-up

Published Date commit `4cbd72b6e86b7f3931cb199c2b0476a9b6b4faa1`
[failed CI](https://github.com/Eriskii/ErisBrowser/actions/runs/36533678216) when stable advanced to Rust 1.98:
both stable jobs stopped at `clippy::chunks_exact_to_as_chunks`. The
minimum-Rust and Vulkan-probe jobs passed; two presenter matrix jobs were
cancelled. The original failure record and logs remain hash-bound in the
[separate follow-up evidence](date-ci-followup.json).

The single source change replaces `rawtypes.chunks_exact(6).enumerate()` with
`rawtypes.as_chunks::<6>().0.iter().enumerate()` in the TZif type-record loop.
The already-checked six-byte records and iteration order are identical. Rust
1.88 supports this API. No parser behavior, timezone policy or quota changed;
the DateHost readiness optimization is not included.

Follow-up adapter SHA-256:
`26f5aa3e7d8440ec652948567f6fa41c5973b3835f77fb51b64e2b4f040e097c`.
Exact production input digest:
`fb9df6d5d132062d3f19e71ed32843abe3439542ab1ade7fefefda632bcd4ed1`.
The source archives verify that this is the only production-source difference
from the original Date input `7bb6ff9b49fa7a66f88cf803a9fec4aee3eb0c4b76d3e14bd5472900c2218a02`.

Local checks pass: Rust 1.98 strict Clippy in default and feature configurations,
Rust 1.88 feature Clippy, all **1,149 Rust 1.98 feature tests**, and the **30
Rust 1.88 Date arithmetic/timezone tests**, with no failures or ignored tests.
The unchanged local fixture again passes **406 modes and twelve controls**.

All **36 formal profiles / 16,146 modes / 3,592 controls** retain identical
complete observations and fingerprints; all **29 existing baseline gates**
pass. Aggregate outcomes remain **13,939 passed / 70 failed / 1,999 unsupported /
138 resource stops**. Date still retains four for-of parser failures and 22
metadata exclusions. No baseline, fixture expectation, policy or published
evidence was updated. Earlier pixel/HTML/stress/timing evidence was not rerun
for this iteration-only change.

Follow-up remote CI is pending publication; these local results do not claim
that the new remote run is green.
