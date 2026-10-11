# Shorthand getters with variable longhands

When each `margin-*` longhand contains `var(--m)`, `getPropertyValue("margin")`
now returns an empty string. Direct longhand reads retain their specified tokens
and priorities. An original `margin: var(--m)` declaration still reconstructs its
source. `removeProperty("margin")` returns the same empty getter result for the
explicit longhands and still removes all four sides.

The [CSS Values substitution rule](https://drafts.csswg.org/css-values-5/#substitution-in-shorthands)
requires this distinction: concatenating unresolved components could change the
meaning when the resulting shorthand is parsed. The
[CSSOM removal algorithm](https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-removeproperty)
returns the prior getter value independently of whether declarations were removed.
These primary sources were checked on 2026-10-11.

The getter uses the existing token-aware variable scanner after checking complete
original pending shorthands. Nested and escaped function names are recognized;
comments, strings and URL payloads do not spuriously count as variable functions.
This covers the currently supported shorthand routes, not every substitution
function or all CSS properties.

For ordinary complete, equal-priority shorthand components, the existing prepaid
work charge increases from `4V + 1` to `8V + 1`, where `V` sums component lengths
plus one per component. The whole charge precedes scanning. This applies to literal
components too; original pending shorthand queries retain their prior charges.
No retained metadata, allocation cap, type layout or renderer command changes.
An early variable rejection skips later background serialization charges, so total
background-query cost need not increase in every case.

Eight unit groups check observable values, token boundaries, removal and exact
work limits. Two integrations exercise page scripts and the confined worker across
three edit states and two viewport widths. Literal pixel and hit-test controls
precede title assertions. Before implementation, both integrations passed their
initial pixel controls and failed the empty-getter assertion; the unit baseline
passed two controls and failed the other six groups as predicted. No existing
assertion was weakened.

Rust 1.88 passed all 3,056 tests with `vulkan-raster` and 2,752 with default
features, including ignored tests. Strict all-target Clippy passed on Rust 1.88
and 1.98; formatting passed. All 102 self-authored reference pairs passed, and
all 204 PNGs are byte-identical to the parent archive. Native integration witnesses
interpret packed plans on the CPU. The source inventory binds 745 inputs; the
browser and checks used that same inventory. Build caches were reused, so these
run times are not cold-build measurements.

The parent `ccde5c04` passed all nine [GitHub checks](https://github.com/Eriskii/ErisBrowser/actions/runs/38161736591).
This increment adds no GPU execution, window acquisition, performance measurement
or broader conformance claim. Existing renderer and security limitations remain.

The [evidence summary](evidence/cssom-var-getter/summary.json) binds the source,
checks and [retained archive](evidence/cssom-var-getter/sources-and-checks.tar.gz).
