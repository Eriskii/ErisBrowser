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

Successful initialization retains its original field values, installation order,
Date host assignment, work charges and heap charges. It leaves **70,334 work
units** before any script-entry reset. Numeric quotas and all existing script
and event reset points are unchanged. This change adds no JavaScript or DOM
feature support. A [later DataView startup optimization](evidence/data-view-bootstrap-lookup.json)
removes 21 unused prototype lookups and increases the remaining work to 70,796;
heap charges, object counts and numeric limits stay the same.

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
