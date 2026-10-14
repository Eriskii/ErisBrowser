# Runtime bootstrap stack regression

The Promise foundation commit increased the inline runtime state by 192 bytes.
GitHub CI on Rust 1.99 then overflowed the existing 128 KiB thread in
`thirty_two_activations_execute_on_small_native_stack_without_parsing_there`.
The same failure reproduced locally on Rust 1.98. A debugger located it during
DataView metadata initialization, before script execution or Promise installation.

`try_new` now initializes one local runtime through a borrowed helper. This
removes whole-runtime return temporaries from the nested initialization path.
The installer order, original errors, heap charges, bootstrap work allowance,
author budget reset and existing tests are unchanged. A test-only consuming
wrapper preserves the raw-bootstrap accounting fixtures.

The unchanged small-stack witness passes. Full suites pass on Rust 1.88 with
default features (2,844 tests), and Rust 1.98 and 1.99 with `vulkan-raster`
(3,148 tests each), including confined workers. This establishes those tested
builds and paths; it is not a measurement of exact stack savings or a guarantee
for every compiler and target.

[Evidence](evidence/runtime-bootstrap-stack/summary.json) includes the source
inventory, original failure and debugger trace, CI compiler identification,
source review and validation logs.
