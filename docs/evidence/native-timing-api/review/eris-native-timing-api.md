# Native timing API contract proposal

Source-only contract against published `f945b61958b61a2f2702d699112e1158bbb46b41`.
Contract sealed before Rust implementation; subsequent implementation is separately authorized. The design keeps the existing owner, deadlines,
queue-write cleanup, single pending packet and release acknowledgment.

## Shared public types and facade

Put the small platform-neutral types in a new `src/presenter/timing.rs`, exposed
within the binary through `presenter`. Use no new dependency:

```rust
struct TimingRequest {
    id: u8,                        // exactly 1..=benchmark_frames, never reused
    preparation_started: Instant,  // browser supplies immediately before paint/prepare
}
enum TimingRoute { CpuUpload, NativeRaster }
enum TimingOutcome { Presented, NotPresented, Aborted, Failed }
enum TimingSubmission { None, Draws, Flush }

struct TimingSample {
    id: u8,
    stamp: FrameStamp,
    size: (u32, u32),
    route: TimingRoute,
    outcome: TimingOutcome,
    submission: TimingSubmission,
    configured: bool,
    ui_prepare_ns: u64,
    queue_ns: u64,
    acquire_ns: Option<u64>,
    encode_upload_ns: Option<u64>,
    submit_ns: Option<u64>,
    completion_wait_ns: Option<u64>,
    cleanup_ns: Option<u64>,
    present_call_ns: Option<u64>,
    owner_total_ns: u64,
    prepare_to_present_ns: Option<u64>,
}
struct TimingCompletion { id: u8, stamp: FrameStamp, outcome: TimingOutcome }
struct TimingInit { owner_init_ns: u64 }
struct TimingReport {
    requested: u8,
    accepted: u8,
    initialization: Option<TimingInit>,
    samples: Vec<TimingSample>,
    failure: Option<String>, // existing bounded failure, not per-record text
}
```

Records and completion values contain no strings, buffers, references or
`Instant`s. Derive `Copy` where possible. Keep failure text in the existing
bounded diagnostic channel. All listed struct fields are `pub(crate)`; the types are reexported by
`presenter`. The enum variants are stable for the first host grammar. Missing
phases are `None`, never invented zero
durations. Checked conversion to nanoseconds must not wrap. Record statuses
describe actual reached work; a successful queue flush is not a drawn frame.

Add `PresenterConfig::benchmark_frames: u8` (0 disabled, 2..=128 enabled),
and `benchmark_check: bool` (default false). Reject
timing with `verify_frames != 0`, software presenter, headless, screenshots or
other benchmark modes before window/worker startup. Require a `vulkan-raster`
build for both timing routes so the paired executable and canonical scene
definition are identical. `native_raster=false` remains a supported CPU-upload
timing route. Repeat validation at `Presenter::new` and `browser::run`.

`benchmark_check=true` is mutually exclusive with nonzero benchmark_frames,
requires the same Linux/vulkan-raster/Vulkan eligibility and exactly one
verification frame. It allocates no timing records and performs no additional
timing clock reads. Browser uses the same canonical identity and fixed status
for a separate correctness process; the ordinary verification owner path
retains its original GPU verification operations. Check-only startup/loading
packets cannot consume verification: arm_benchmark_check requires the idle,
unarmed owner and binds the next accepted, route-matching packet serial. It
arms once; any additional packet is fatal. benchmark_check_complete requires
that exact serial's verification plus packet drop/idle, no error, and no stop.
One opt-in idle wake delivers this boundary. A readiness call made while
startup is busy also registers one idle-interest wake (both timing/check modes),
so the first fixed-scene request need not wait for a deadline. Default idle
adds no wake.

Keep existing `submit` and `submit_native` signatures/behavior; route internally
through a common optional-metadata helper. Add only:

```rust
fn timing_ready(&self) -> bool;
fn submit_timed(&mut self, canvas: Canvas, stamp: FrameStamp,
                timing: TimingRequest) -> Result<Submission, String>;
fn submit_native_timed(&mut self, plan: Plan, stamp: FrameStamp,
                       timing: TimingRequest) -> Result<Submission, String>;
fn take_timing_report(&mut self) -> Result<TimingReport, String>;
fn arm_benchmark_check(&mut self) -> Result<(), String>;
fn benchmark_check_complete(&self) -> bool;
```

The native timed signature deliberately has no reference argument. Timed
methods reject disabled timing, route mismatch, non-next sample IDs or a second
outstanding sample. They retain current serial/epoch checks. A returned status
other than `Queued` aborts the browser benchmark; it does not retry that sample.

Extend `Notice` with `timing_completion: Option<TimingCompletion>`. Do not set
its existing `redraw` bit on timing completion: the browser owns whether to
request another sample. `timing_ready` means initialized, current usable
surface owner, phase Idle, no active/pending packet, no undelivered completion,
no failure/stop, and quota remaining. Acceptance rechecks these under the lock.

In timing mode, the existing `Presenter::service` automatic software takeover
must be disabled, and `Worker::service/finish` must treat failure or incomplete
sample count as an error, just as verification currently does. The ordinary
non-timing fallback policy is unchanged. `take_timing_report` is a one-shot
move after positive `Phase::Released`; failed reports remain extractable with failure text,
but a stalled owner or finished thread is
insufficient. Retain partial records on failure for the host to report, but
successful benchmark exit requires exactly N presented samples and no failure.

## Packet, owner and completion sequence

Add `Packet::timing: Option<PacketTiming>`; ordinary constructors set None.
`PacketTiming` contains the request and an acceptance `Instant` set under
`Shared::submit`, only for a timed packet. The interval from preparation start
to acceptance includes route preparation, packet validation and normal mutex
admission; document it as such. Move optional scene logging outside this bracket
in timing mode; keep regular rendering diagnostics unchanged.

On `Shared::take`, construct owner-local `Option<TimingTrace>`. A trace records
the taken time and optional reached timestamps. Each extra clock read must be
inside `if let Some(trace)`/lazy `Option::map`, never eagerly evaluated for None.
Existing timeout `Instant` calls remain; disabled timing adds no new ones,
telemetry allocation, frame-completion wake or repeated redraw.

Instrument existing brackets without moving GPU operations: acquire/configure,
CPU conversion/write_texture or native allocation/encode/queued writes,
encoder finish/submit, tracked poll, scope resolution/resource cleanup and
present call. `prepare_to_present_ns` is captured only after a successful
present call and its existing failure check. `owner_total_ns` extends through
packet drop and idle transition. The durations are host wall time, not GPU
timestamp or display-latency measurements. No reference/readback is introduced.

The owner loop becomes structurally:

```text
take packet; make optional trace
result = gpu.frame(packet, shared, verify, optional trace)
if result is an error:
    retain partial failure trace; drop packet
    take the existing fail / begin_release / drop(gpu) / released path
    do not issue a completion or redraw permit
else:
    finish frame-owned resources as today
    drop packet
    shared.idle()                         // existing deadline check stays here
    publish trace and completion, if enabled and state still permits it
```

`gpu.frame` may return `Ok(())` after skipped acquisition, obsolete work or a
retired cancellation; only an explicit successful present marker means
`Presented`. Such non-presented records may be published after safe idle but
must terminate the benchmark, not trigger another draw. On GPU error, do not
call idle early merely to deliver telemetry or clear active GPU accounting.
Keep a failure record without completion; final report remains failed.

`Shared::complete_timing` appends once and exposes one completion only after
packet drop and the idle check; it refuses duplicates, wrong ID/stamp, record
overflow or a state that expired while completing. Notify outside the mutex.
It may reuse the existing wake coalescing, but never the graphics `ready`
condition variable as a UI completion permit. Preserve all `MayHaveWrites`
flush decisions, discarded partial encoders, tracked retirement, late error
scopes and original deadline values exactly. Telemetry failure is terminal
only after required graphics cleanup, never an early return around it.

Before the first timed request, untagged startup/loading packets retain their
normal handling. After timing starts, the browser suppresses incidental redraw
submission while a sample is outstanding; a second timed or untagged packet
must not replace it unnoticed. If invalidation drops a pending timed packet,
retain accepted/incomplete accounting and fail the run; do not synthesize a
completed duration for work the owner never took.

## Bounded retention and ledger

`State` owns `Option<TimingState>` with a single `Vec<TimingSample>`, counters
and one pending completion. Reserve N records fallibly before owner startup;
check actual capacity and reject capacity above128. No resizing or record clone
is allowed during frames. Include
`size_of::<TimingState>() + capacity*size_of::<TimingSample>()` in
`State::live_bytes()` through a checked `benchmark_bytes` term. Packet timing adds
its fixed metadata size to timed packet capacity accounting. Reserve a fixed 256KiB
identity/metadata and owner-trace/notice handoff allowance too; document stack/allocator exclusions
consistently with the current explicit ledger rather than claiming RSS bounds.

The original 128MiB ceiling and 16MiB next-UI-frame reserve do not increase.
Validate the timing reserve before allocation, then verify actual capacity;
all later scratch/native reservations see the timing charge. The report moves
out after owner release, so no second 128-record copy overlaps active rendering.
UI serializes it once with bounded output. Every attempted accepted sample
must have either one terminal record or an explicit incomplete-run failure;
no dropping samples to make a distribution look successful.

## Browser fairness and exact input identity

Root/browser ownership should add a `NativeTimingRun` state machine: waiting
for loaded snapshot/settled size/idle owner → armed → one outstanding sample
→ completion → next sample or close. It passes only `TimingRequest` to the
presenter; scene state and correctness are browser/host responsibilities.
An event changing navigation, snapshot, viewport, zoom, scroll, selection,
focus or paint-relevant chrome after arming fails the run. Owner completion
never advances the UI by itself.

Use one explicit benchmark-only status override, for example the fixed ASCII
string `Native timing scene`, in **both** `paint_canvas` and native scene build.
Factor the selection of this override into a common helper; do not mutate
`snapshot.load_ms` or separately format its value to zero. Preserve real
`load_ms.to_bits()` and diagnostics in run metadata outside the paint identity.
Do not let the override conceal worker errors or CPU paint exhaustion: either
still fails the timed run. Outside benchmark mode the original status branches
and text remain unchanged.

Bind a canonical **paint-input byte sequence**, not command counts, a fast
unkeyed hash, or pointer identity. A small std-only tagged/length-prefixed writer
can serialize once before timing:

- Physical target/scale and canonical clear; ordered page/overlay/chrome phases
  with exact f32 bits for clips and document/fixed offsets.
- Every original DrawCommand in order, all geometry/flags/colors, exact UTF-8
  text/image keys and every scope operation, including hidden commands.
- Sorted image keys and deterministic alias IDs based on first appearance in
  sorted-key order; unique image dimensions and complete RGBA bytes. Distinct
  equal-content Arcs remain distinct sources; numerical pointer addresses are
  never serialized.
- Exact resolved title/address/status and normalized selection/focus/scroll/
  zoom inputs, content height and relevant focus geometry. Freeze the normalized
  selection before either route; include all paint-relevant state in the shared
  definition, not a handpicked count signature. Bundled font identities and
  complete renderer source/fixture bindings belong in host provenance.

Expose a browser-local canonical-scene helper using the same `native_scene`
phase construction but **no** plan, mask rasterization or Canvas. Its output
describes common paint inputs; separate correctness runs still establish the
CPU/native output equivalence. Normalize only the explicitly overridden status
input. Store volatile transport/run facts, including generation, edit sequence,
task state, raw load time and diagnostics separately rather than silently
discarding them from the retained record.

Use a fixed identity ceiling of64KiB with a counting pass before a
fallible exact reserve. Apply existing 256-command/source,512-store-capacity,
key/text/image byte bounds before sorting or copying. Refuse an oversized
identity rather than truncate it. Charge a fixed 256KiB benchmark identity/metadata allowance before startup,
plus actual sample-vector capacity. The browser identity implementation uses
no more than two64KiB canonical buffers; this allowance also covers bounded
serialization/notice/trace metadata. No separate browser reservation API is
needed; failure must precede identity allocation. Streaming comparison against retained bytes avoids a
second full identity copy. For this static fixture, reject a replaced snapshot
or changed UI token and check exact canonical equality outside each timed span;
do not rely solely on a token to prove equality.

Emit the complete canonical bytes once to a bounded host artifact before the
sample sequence. The host parses the versioned structure, compares CPU/GPU
bytes exactly and retains them; SHA256 is an artifact binding, not a substitute
for this comparison. Reuse one listener and **identical URL/port** for paired
CPU/GPU processes so the address-bar glyph stream stays equal. Use identical
viewport, title, override, focus and other frozen UI state. Header/provenance
generation is outside sample clocks, and repeated frames still rebuild their
real route's plan or CPU target inside those clocks.

## Coordination and tests

Presenter owner: timing types/config propagation, packet metadata, guarded
clocks, ledger/reservation, owner records/completion and fatal benchmark errors.
Browser owner: CLI/state machine, common status override, canonical inputs,
stable-scene checks and timed submission calls. Host owner: paired listener,
release/source/asset bindings, exact canonical comparison, strict raw records,
cleanup and summaries. Existing worker and native host correctness protocols
remain independent.

Before actual measurement, synthetic tests should prove zero optional clock
reads/allocations/wakes when disabled; sequential IDs; one completion after
drop and idle only; no advance after skip/flush/error; late-scope/deadline failure
preservation; bounds/overflow on records and identity; benchmark+verification
rejection; no software recovery counted as timing success; equal status bytes
despite different real load times; alias-aware identity difference detection;
and exact host identity/phase/count rejection. No new GPU capability or relaxed
resource/deadline policy is needed for this API.
