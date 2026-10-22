# CSS pointer-events: auto and none

`pointer-events:none` excludes a box from the existing point-target search
without removing its painting or layout geometry. Content behind it can become
the target. The inherited default is `auto`; a descendant can explicitly restore
`auto` inside a `none` ancestor. The ancestor remains on the descendant's event
path, so capture and bubbling still work. These are the HTML `auto`/`none`
behaviors described by [CSS UI 4](https://www.w3.org/TR/2026/WD-css-ui-4-20260120/#pointer-events-control),
a Working Draft whose broader hit-testing and selection details remain open.

Values use one decoded, ASCII-insensitive identifier. CSS-wide keywords,
variables, layer/importance ordering and the `all` shorthand follow the existing
cascade. Invalid later declarations do not replace an earlier valid declaration;
an invalid computed variable value uses the inherited property's `unset`
behavior. Inline `style.pointerEvents`, `setProperty`, removal and getters use
the bounded CSSOM machinery. The supported longhand roster now has 88 entries.

Feature queries share the keyword validator. A condition can decode an escaped
CSS property name, while the two-argument `CSS.supports(property,value)` API
keeps its literal-name rule. The subsequent [variable-query slice](css-supports-variables.md)
admits valid legacy `var()` syntax without substitution, superseding the earlier
conservative false result. Valid references defer final property validation under
[CSS Variables](https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#using-variables).
The validation results below retain their original pointer checkpoint scope.

Layout retains every admitted hit region, its rectangle and its order. The
ordinary-box, default-summary, text and table-row producers record their own
computed pointer value. Direct layout and native browser point selection skip
`none` and continue through the previous paint-order search. Existing clipping,
fixed/scroll coordinates and zoom rules remain in force.

Keyboard focus, editing reconciliation, fragment scrolling and native focus
outlines continue to use all retained geometry. A pointer setting does not clear
focus or suppress keyboard activation. Node-targeted event dispatch and DOM
ancestry are unchanged. `visibility:hidden` still excludes its own hit regions;
`pointer-events:auto` does not revive them. `display:none`, canvas-background
selection and all painting behavior remain unchanged by this checkpoint.

ERWC carries one additional 0/1 byte after each hit action: `0` for `auto`, `1`
for `none`. Each serialized hit is 23 bytes, independent of Rust structure
padding. `none` records remain valid because other consumers need their geometry.
Unknown values and old protocol versions are rejected; previous node, visibility,
rectangle and action checks remain. There is no additional per-node table.

The existing limits remain: layout creates at most 100,000 hits, while the worker
decoder retains its separate 200,000-hit ceiling and existing frame cap. The new
wire cost is one byte per hit. CSS raw-value, work, 256-record and 65,536-byte
logical-storage limits are unchanged; `all` fixture arithmetic accounts for the
larger roster.

The default Rust 1.88 suite passes 3,577 tests; Rust 1.99 with
`test262-host,vulkan-raster` passes 3,942. Both include ignored tests explicitly,
with zero failures or skips across 62 targets. Their prior passing rosters are
preserved with 42 new default groups and one extra native-raster group. Strict
Clippy passes for both configurations; formatting and the release build pass.
All 102 existing visual-reference pairs pass, and their 204 PNGs are byte-identical
to the previous canvas-display checkpoint.

The original parent passes three of the 19 public targeting/CSSOM/dynamic groups
and fails the other 16. The final candidate passes all 42 focused groups,
including the three additional all-reset regressions. Direct and confined-worker
dynamic controls each cover 24 full-frame snapshots (scripts off/on, six phases,
two widths). Feature-enabled runs compare the same pixels with the CPU native
scene witness; they are not GPU hardware executions.

Candidate1 exposed an `all:initial` CSSOM work-budget failure. The final change
skips redundant name searches only for an `all` expansion into an empty block,
whose static longhand names are distinct. Record admission remains before every
push; nonempty merges, importance, source order, summary/clone/commit accounting
and limits stay in place. The unchanged public control now passes. Three new
controls preserve earlier priority, later order and pending shorthand behavior;
before the correction, the two bounded parse controls failed and the merge
control passed. All failed attempts and the additive correction are retained.

Measured on the tested x86_64 Rust 1.88 build, `ComputedStyle` remains 1,032 bytes,
`HitRegion` 32 bytes and `LayoutResult` 80 bytes, all aligned to 8. The new byte
fits existing Rust padding, so hit-vector stride does not grow. ERWC still adds
one serialized byte per hit; the Rust layout result is platform-specific.

Local validation is complete for this checkpoint. GitHub CI for its new commit
is pending at publication; the preceding `3d90bd57` checkpoint passed all 11 jobs.
See the [validation manifest](evidence/pointer-events/validation.json) and
[source/check archive](evidence/pointer-events/sources-and-checks.tar.gz).

The controls cover target changes alongside unchanged geometry/pixels, inherited
and re-enabled regions, event paths, focus, exact grammar budgets and malformed
transport. They are bounded implementation regressions. SVG-specific values,
pointer capture, specialized pointer events, touch/drag selection and complete
hit-testing conformance are not added. There is no GPU validation, new benchmark,
Chromium speed comparison, whole-browser or production-security claim.
