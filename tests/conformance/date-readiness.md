# Date helper readiness waits

Timezone discovery now waits for readable response bytes and, while needed,
writable request space. The previous unconditional I/O sleep imposed about
10 ms of startup cost. Polling is capped by the existing absolute deadline and
cancellation interval. Buffered bytes survive a hangup; EOF alone does not
establish a successful helper exit. The service still owns the child through
group termination and reaping, and rejects another capture during cleanup.

The [published parent](https://github.com/Eriskii/ErisBrowser/commit/a978af13dc0bc2766d05fda8033a03606ba1208e)
has [passing CI](https://github.com/Eriskii/ErisBrowser/actions/runs/36535049402).
It includes the separately documented [Rust 1.98 correction](date-ci-followup.md);
the earlier failed run and original Date evidence remain unchanged.

The [worker comparison](../../docs/benchmark-worker-date-readiness.json) uses
nine fixtures, three fresh workers per fixture and twenty warm frames per run,
separately for default and feature-enabled binaries. Startup median differences
range from **−9.273 to −9.005 ms** by fixture in the default build, and
**−8.485 to −8.205 ms** in the feature build. After-change startup medians range
from 2.039–2.207 ms and 3.626–3.924 ms respectively. Warm-frame median differences
range from −0.079 to +0.102 ms and −0.092 to +0.366 ms. All eighteen final fixture
PNG pairs are byte-identical. These are sequential local measurements with
uncontrolled OS caches; the feature runs are headless and do not exercise GPU
presentation. They establish no Chromium comparison.

The [validation record](date-readiness.json) binds compiler inputs captured
before both release builds, binaries, reports and logs. Rust 1.98 passes
**1,144 default tests and 1,155 feature tests**, with no failed or ignored tests;
strict Clippy passes in both configurations and in the Rust 1.88 feature build.
Six new host test groups cover readiness, fragmented output, partial writes,
interruptions, cancellation, deadlines and EOF before process exit. The private
partial-write stress input deliberately exceeds the public configuration cap;
the cap itself is unchanged and remains tested.

Both builds pass all 57 existing pixel references. The unchanged local Date
fixture passes 406 modes and twelve controls. All 36 formal profiles retain
identical observations across 16,146 modes and 3,592 controls; all 29 existing
baseline gates pass. No fixture, policy or baseline changed. The DOM and stress
executables are byte-identical to the parent builds, so their previous results
are retained without another execution.

A race-dependent delay remains when output closes before exit becomes
observable: that separate exit wait is capped by the remaining deadline.
No pidfd or new dependency is introduced. Timeout still does not imply a
blocked kernel operation or cleanup has completed. Full compatibility,
security assurance and the overall browser performance requirement remain open.
