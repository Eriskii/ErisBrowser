# Call and constructor argument grammar contexts

Call and constructor arguments accept `in` expressions even when the argument
list appears in a `for` initializer. For example:

```js
for (consume("key" in object); false;) {}
for (new Box("key" in object); false;) {}
for (consume(..."key" in object ? [1] : []); false;) {}
```

ECMAScript's ArgumentList grammar enables `In` for each assignment-expression
operand. The parser previously inherited the initializer's restriction instead.
Each nonempty argument continuation now saves the outer flag, enables it for its
operands, and restores it before publishing the call/new expression. Nested
argument lists own their own saved flags. Both closing-parenthesis paths,
including trailing commas, restore the context. Empty lists and constructors
without parentheses retain their existing behavior.

On parser errors, the continuation driver restores its invocation-entry `In`
flag. It preserves the original error, token position and spent quotas. This is
not full parser rollback or a guarantee that failed parsers can be reused.
Successful dynamic-function parameter parsing still carries its active function
context into body parsing; function completion restores the outer context.

The change uses the existing argument frames and grammar dispatch charges. It
adds no runtime fields, allocations, syntax records, dispatches or resource-limit
changes. The reference parser used by differential tests implements the same
argument-context rule.

All 25 new regression groups pass: 15 public interpreter/parser groups and ten
private context, error and accounting groups. Before the fix, the public groups
reported six passes and nine failures; the runnable private controls reported
four passes and six failures. The retained evidence records the two test-helper
corrections separately from implementation changes.

Complete suites pass **3,024 tests on Rust 1.88** and **3,378 with
`test262-host,vulkan-raster` on Rust 1.99**, each across 50 result rows with no
failed, ignored or filtered tests. Strict Clippy passes for all targets in both
configurations, and formatting passes. On x86_64 Linux, actual parser expression-frame,
driver-kind and transition sizes remain 72, 112 and 224 bytes. Fourteen existing
full-compile work/heap observations remain exact in both configurations.

The release adapter preserves every case and harness record in the pinned
196-mode `Promise.all` and 458-mode Promise profiles, including all 90 and 74
harness controls. No conformance baseline changes are needed. These replays
check preservation; the new grammar cases are covered by the focused tests.

The [evidence record](evidence/call-argument-in/summary.json) includes changed
source files, frozen tests, parent observations, reviews and execution receipts.
It binds the complete build-input inventory to the parent Git commit rather
than duplicating the unchanged repository in the archive.

This correction does not complete ECMAScript grammar, web compatibility,
production security or the Chromium-relative performance target. The later
[computed-member and conditional-expression follow-up](expression-in.md) handles
their separate grammar contexts and comma expressions inside brackets.

Specification: [Argument lists](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-argument-lists).
