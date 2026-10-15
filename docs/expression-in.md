# Computed keys and conditional grammar contexts

Computed member keys accept comma expressions and relational `in`, including
inside `for` initializers. The middle operand of a conditional expression also
enables `in`; its test and false operand retain the surrounding grammar context.
For example:

```js
for (object["key" in source]; false;) {}
for (object[recordAccess(), "key"]; false;) {}
for (true ? "key" in source : false; false;) {}
```

Computed brackets in member access, call results and constructor selection use
the existing sequence-expression parser. A single operand retains its original
expression identity. Comma operands evaluate from left to right and use the
existing sequence record; a trailing comma remains invalid. The conditional
middle remains an assignment expression, so an unparenthesized comma there is
still invalid.

Each owner saves and restores the surrounding `In` flag. Closing a bracket
restores it before the fallible member emission. Consuming a conditional colon
restores it before scheduling the false operand. Nested owners keep their own
flags. The reference parser used by differential tests follows the same rules.
The existing driver restores its invocation-entry flag on errors while retaining
the reached token position, error and spent budget; it does not roll back the
whole parser.

The flag changes add no new fixed charges. Computed keys pay the existing
sequence dispatch, list and record charges. Even a single key adds sequence
dispatches, and the additional pending continuation can change stack-storage
growth or reach the unchanged continuation limit earlier. There is no universal
fixed work or heap delta. Runtime limits, startup allowance and AST
representations are unchanged.

All **31 new regression groups** pass: 21 interpreter/differential-parser
groups and ten internal context, error and accounting groups. The public groups
cover both strict modes. Before the fix, the public baseline had seven passes
and fourteen failures; the private baseline had four passes and six failures.
No frozen expectation or existing test needed adjustment.

The complete suites pass **3,055 tests on Rust 1.88** and **3,409 with
`test262-host,vulkan-raster` on Rust 1.99**, each across 50 result rows with no
failed, ignored or filtered tests. Strict Clippy passes for all targets in both
configurations, and formatting passes. Actual x86_64 Linux expression-frame,
driver-kind and transition sizes remain 72, 112 and 224 bytes. Three
conditional-only full-compile work/heap pairs remain exact in both modes and
configurations. Computed-key costs use the existing Sequence charges and are
not asserted unchanged.

The release adapter preserves all case and harness records in the pinned
196-mode `Promise.all` and 458-mode Promise profiles, with all 90 and 74 harness
controls passing. These replays check preservation; no conformance baseline is
changed. The new grammar cases are covered by the focused tests.

The [evidence record](evidence/expression-in/summary.json) binds the changed
build inputs to the parent commit, with the complete source inventory, frozen
regression controls, parent observations and execution receipts.

This grammar correction does not establish full web compatibility, production
security or the Chromium-relative performance target.

Specifications: [Computed member access](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-left-hand-side-expressions),
[conditional expressions](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-conditional-operator),
and [comma expressions](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-comma-operator).
