# Promise.prototype.catch

`Promise.prototype.catch` forwards through an observable lookup and call of
the receiver's `then` method. It passes exactly `undefined` and the original
rejection argument, preserves the original receiver, and returns the method's
result unchanged. This works with generic objects and supported primitive
receivers. Getter/call exceptions retain their original JavaScript values.

For genuine Promises using the ordinary `then`, rejection handlers enter the
existing shared microtask FIFO. Their work and allocations use the same limits
as synchronous code and other reactions. The wrapper does not create another
checkpoint or reset a work budget.

Seven semantic test groups cover generic invocation, primitive receivers,
unfiltered arguments, abrupt completions, queued reactions, observable method
replacement and function metadata. Page and confined-worker tests additionally
verify disabled-script controls, initial jobs, repeated polling, click-listener
ordering, pixels and hit testing. Rendering assertions use fixed-coordinate
procedural expected frames; twelve archived RGBA frames were separately checked
against that same geometry. The tests do not load those archived files. These nine groups failed on the
foundation parent before implementation. Later assertions in a failing group
were not executed in that baseline.

Installation adds one metadata object and 102 startup work units. Raw startup
uses 99,907 of the unchanged 100,000 allowance; the full realm has 748 object
slots. Exact and one-short installer/bootstrap tests retain their refusal and
publication assertions. Six existing test files have explicit metadata/count
migrations; new semantic expectations were held before production changes.
The final checks also exposed a raw-startup inspection test that needed its
subsequent metadata lookup separated from initialization work. That correction
retains the raw-budget assertion and models the existing public budget reset.

Final full suites pass with 3,157 tests on Rust 1.99 with `vulkan-raster` and
2,853 on Rust 1.88 with default features, including 88 confined-worker tests
in each. An intermediate parallel default run had an unrelated host-process
timeout; its log is retained, and the unchanged suite passed when run alone.

This remains a partial Promise implementation. `finally`, combinators,
async functions/`await`, rejection notifications, multiple realms and garbage
collection remain unfinished. The pixel checks use CPU painting and CPU
interpretation of native rendering plans; they do not demonstrate GPU output.

[Evidence](evidence/promise-catch/summary.json) retains the frozen expectations,
parent failures, source reviews, startup ledger and validation receipts.
