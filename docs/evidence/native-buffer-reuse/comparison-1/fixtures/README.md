# Native-window inputs

These local pages request a real worker-loaded scene with short text, one
decoded image, a fixed text element and tall content requiring a scrollbar at
1180×880. Scripting is disabled. `admitted.html` uses one propagated white body
background; `opacity.html` adds group opacity and requires whole-frame CPU
fallback.

`overdraw.html` preserves the exact original admitted input, including separate
white HTML and body backgrounds. Its original controlled run reached 1180×880
but refused the unchanged four-million-invocation limit. It now has a separate
`planner-budget` fallback case. The earlier wrong-size and overdraw failures
remain retained observations; this fixture revision does not change limits or
replace those results.

The admitted verification run compares the actual acquired Vulkan texture to
the original Canvas reference for that completed scene. This is a differential
reference, not an independent font or layout pixel oracle. Matching loaded-scene
and acquired-verification serial/generation records are required; startup chrome
alone does not establish page rendering. A separate normal run requires
reference=false. Both fallback cases are admission evidence without a native
pixel comparison. Observation stops before the desktop compositor; no screenshot
or performance claim follows.

`two-pixels.png` remains the unchanged earlier worker-text input. The manifest
binds these exact bytes before their next execution. The single-background
revision follows the preserved original overdraw observation; no new renderer
execution was used to author it.
