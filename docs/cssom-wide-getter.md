# Shorthand getters with mixed CSS-wide values

A margin with `initial` on one side and `1px` on the others now has an empty
shorthand getter. Previously it could return `initial 1px 1px`, which is not a
valid equivalent shorthand. Direct longhand values, priorities and serialization
stay intact. Removing the margin still deletes all four sides and returns the
empty prior shorthand value.

The [CSSOM serialization rule](https://drafts.csswg.org/cssom/#serialize-a-css-value)
requires exact representation of the longhand values. Under
[CSS Cascade](https://drafts.csswg.org/css-cascade-5/#shorthand), a CSS-wide keyword
sets every constituent and cannot be combined with ordinary components.
Uniform `initial`, `inherit`, `unset`, `revert` and `revert-layer` values still
reconstruct their keyword. Parsing already normalizes keyword case and escapes.
These primary sources were checked on 2026-10-11.

The shared shorthand formatter classifies its first component once. A wide first
component allows only identical remaining values. Otherwise, a wide value in any
remaining component prevents synthesis. The existing background serializer and
earlier missing-value, priority, pending-source and variable checks retain their
routing. This changes the current supported shorthand routes; broader property
and serialization support remain incomplete.

The mutually exclusive scans fit the existing synthesis allowance. The getter
still prepays `8V + 1`, where `V` is the sum of component byte lengths plus one
per component. No fee, retained field, allocation limit or renderer command is
added. Exact and one-short getter/removal tests retain the prior transaction
boundaries, including the earlier variable and pending-shorthand controls.

Eight new unit groups cover the current joining routes, all five keywords,
positions, case/escapes, priorities, pending sources, removal and exact work cuts.
Two integrations exercise scripts and the confined renderer across two edit
states and two widths. Initial pixel and hit checks precede getter assertions;
removal then moves the red box while the uniform-keyword blue control stays put.
The parent passed one unit control and failed the other seven groups as predicted.
Both parent integrations passed their initial pixel controls and failed the
mixed-value getter assertion. Existing test expectations remain unchanged.

Rust 1.88 passed all 3,066 tests with `vulkan-raster` and 2,762 with default
features, including ignored tests. Strict all-target Clippy passed on Rust 1.88
and 1.98; formatting passed. All 102 self-authored reference pairs passed, with
all 204 PNGs byte-identical to the parent archive. The checks and final browser
share the same 751-input inventory. Build caches were reused; run times are not
cold-build measurements. Native pixel witnesses interpret packed plans on the
CPU and add no GPU or acquired-window observation.

The parent `f0e9b960` passed all nine [GitHub checks](https://github.com/Eriskii/ErisBrowser/actions/runs/38162897601).

The [evidence summary](evidence/cssom-wide-getter/summary.json) and
[archive](evidence/cssom-wide-getter/sources-and-checks.tar.gz) retain the source,
literal expectations, parent failures and validation results. These checks do
not establish full web compatibility, production security or Chromium-relative
performance.
