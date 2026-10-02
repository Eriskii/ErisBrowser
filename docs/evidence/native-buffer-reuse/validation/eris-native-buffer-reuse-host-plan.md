# Reuse experiment host comparison (proposal, no source edits)

Compare the frozen native baseline executable from source51ca9ed, release
SHA038ffe37d58b0e27189ee3681ec6a84ab183beba9433f4a22cca766e47af08de,
to the future candidate with retired-buffer reuse. Keep the normal renderer CLI
unchanged: both use --presenter=vulkan --raster=gpu and the existing exact-scene
check/measurement modes. No production allocation-policy toggle is needed.

Use the existing owned-launch/gated listener/subreaper machinery. A new bounded
comparison entry point can select a frozen binary/hash for each prescribed case;
never duplicate controller or cleanup code. One listener and exact URL span all
cases. Run baseline-check and candidate-check separately, then three alternating
baseline/candidate pairs with16frames each. This is8processes and96native frames,
plus9,011,200 acquired correctness bytes. All reports remain route=native-raster;
keep baseline/candidate as a separate host variant field. Require exact EBNIv1
bytes, scale, adapter/format/present mode and all direct successful phases across
every process. Each case retains raw logs and cleanup before any distribution.
The existing CPU/native host mode must retain its current default behavior.

Bind baseline source separately from the current tree: create a120-file source
snapshot using the published release-2.json manifest and source51ca9ed, checking
every blob SHA. The known baseline executable must match that manifest's binary
SHA and size. The candidate gets its own compiler/source/assets/feature manifest.
Bind both manifests and every actually imported current host helper. No source
mismatch is silently excused as an old revision. Do not include executables in
the public evidence package. Retain complete raw baseline and candidate runs.

The first/subsequent split remains exactly sample1 vs samples2..16. For each
variant compute direct preparation-through-present and owner-total median/p95;
keep initialization and per-phase summaries separate. Do not discard the early
second-frame submission tail. No rerun may replace a failed attempt; all stay
visible. This compares a narrow implementation change on one static fixture,
not Chromium, broad pages, full frame cadence, compositor latency or throughput.

Separate correctness work must exercise changed equal-sized plans/images and
surface formats through multiple reuse cycles before timing. A single check
frame cannot prove that reused resources overwrite old content. Use the core
probe with exact CPU/literal oracles and allocation/reuse observations, then
validate actual native integration with the same completion/retirement tests.
Do not hide preparatory synthetic frames in the timing path or relabel first
frames to make the candidate look better. GPU clocks are not forced by this
proposal; retain environment snapshots and qualify small-sample conclusions.
