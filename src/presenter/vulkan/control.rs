//! CPU-only ownership and scheduling state. No graphics API call under this lock.
use super::super::{FrameStamp, Submission, Target};
use eris::graphics::Canvas;
use std::{
    fmt,
    sync::{Arc, Condvar, Mutex, MutexGuard},
    time::{Duration, Instant},
};

pub(super) const TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const MAX_PIXELS: usize = 4_194_304;
pub(super) const PIXEL_BYTES: usize = MAX_PIXELS * 4;
const APPLICATION_BUDGET: usize = 128 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Initializing,
    Idle,
    Preparing,
    Acquired,
    #[cfg(feature = "vulkan-raster")]
    Encoding,
    UploadQueued,
    Submitted,
    Retiring,
    Releasing,
    Released,
}

pub(super) struct Packet {
    pub stamp: FrameStamp,
    pub size: (u32, u32),
    pub payload: Payload,
    retained_bytes: usize,
}
pub(super) enum Payload {
    Cpu(Vec<u32>),
    #[cfg(feature = "vulkan-raster")]
    Native {
        plan: eris_raster_core::Plan,
        reference: Option<Vec<u32>>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VerifiedRoute {
    CpuUpload,
    #[cfg(feature = "vulkan-raster")]
    NativeRaster,
}
impl Packet {
    pub fn from_canvas(mut canvas: Canvas, stamp: FrameStamp) -> Result<Self, String> {
        super::super::CpuFrame::from_canvas(&canvas)?;
        let bytes = canvas
            .pixels
            .capacity()
            .checked_mul(4)
            .ok_or("pixel capacity overflow")?;
        if canvas.pixels.len() > MAX_PIXELS || bytes > PIXEL_BYTES {
            return Err("Vulkan frame exceeds the four-megapixel capacity budget".into());
        }
        Ok(Self {
            stamp,
            size: (canvas.width, canvas.height),
            payload: Payload::Cpu(std::mem::take(&mut canvas.pixels)),
            retained_bytes: bytes,
        })
    }
    #[cfg(feature = "vulkan-raster")]
    pub fn from_native(
        plan: eris_raster_core::Plan,
        reference: Option<Canvas>,
        stamp: FrameStamp,
    ) -> Result<Self, String> {
        use eris_raster_core::surface::SurfaceLayout;
        // The two admitted native formats have identical storage/work. Repeat
        // against the actual selected format on the graphics owner.
        let layout = SurfaceLayout::for_plan(&plan, wgpu::TextureFormat::Bgra8Unorm)?;
        let size = (layout.width(), layout.height());
        let reference = match reference {
            Some(mut canvas) => {
                super::super::CpuFrame::from_canvas(&canvas)?;
                if (canvas.width, canvas.height) != size {
                    return Err("native reference dimensions differ from plan".into());
                }
                Some(std::mem::take(&mut canvas.pixels))
            }
            None => None,
        };
        let reference_bytes = reference
            .as_ref()
            .map_or(Some(0), |pixels| pixels.capacity().checked_mul(4))
            .ok_or("native reference capacity overflow")?;
        let retained_bytes = plan
            .retained_cpu_bytes()?
            .checked_add(reference_bytes)
            .ok_or("native packet capacity overflow")?;
        if retained_bytes > PIXEL_BYTES {
            return Err("native packet exceeds the reserved next-frame capacity".into());
        }
        Ok(Self {
            stamp,
            size,
            payload: Payload::Native { plan, reference },
            retained_bytes,
        })
    }
    fn bytes(&self) -> usize {
        self.retained_bytes
    }
    pub fn cpu_pixels(&self) -> Option<&[u32]> {
        match &self.payload {
            Payload::Cpu(pixels) => Some(pixels),
            #[cfg(feature = "vulkan-raster")]
            Payload::Native { .. } => None,
        }
    }
    pub fn is_native(&self) -> bool {
        #[cfg(feature = "vulkan-raster")]
        {
            matches!(&self.payload, Payload::Native { .. })
        }
        #[cfg(not(feature = "vulkan-raster"))]
        {
            false
        }
    }
}

pub(super) struct State {
    pub target: Target,
    pub phase: Phase,
    pub deadline: Option<Instant>,
    pub stop: bool,
    pub stalled: bool,
    pub failure: Option<String>,
    pub diagnostic: Option<String>,
    pub redraw: bool,
    pub wake_pending: bool,
    pub verified: u8,
    pub verified_bytes: u64,
    pub last_verified_serial: u64,
    native_raster: bool,
    #[cfg(feature = "vulkan-raster")]
    native_presented: bool,
    pending: Option<Packet>,
    active_bytes: usize,
    scratch_bytes: usize,
    readback_bytes: usize,
    active_gpu_bytes: usize,
    last_serial: u64,
    /// Acquisition may report occlusion without a prior winit event. Only a
    /// subsequent explicit visibility/size update resumes it, not new frames.
    acquisition_occluded: bool,
}
impl State {
    fn new(target: Target, now: Instant, native_raster: bool) -> Self {
        Self {
            target,
            phase: Phase::Initializing,
            deadline: Some(now + TIMEOUT),
            stop: false,
            stalled: false,
            failure: None,
            diagnostic: None,
            redraw: false,
            wake_pending: false,
            verified: 0,
            verified_bytes: 0,
            last_verified_serial: 0,
            native_raster,
            #[cfg(feature = "vulkan-raster")]
            native_presented: false,
            pending: None,
            active_bytes: 0,
            scratch_bytes: 0,
            readback_bytes: 0,
            active_gpu_bytes: 0,
            last_serial: 0,
            acquisition_occluded: false,
        }
    }
    fn live_bytes(&self) -> Option<usize> {
        // Reserve another maximum frame for simultaneous UI painting/admission.
        PIXEL_BYTES
            .checked_add(self.active_bytes)?
            .checked_add(self.pending.as_ref().map_or(0, Packet::bytes))?
            .checked_add(self.scratch_bytes)?
            .checked_add(self.active_gpu_bytes)?
            .checked_add(self.readback_bytes)
    }
    fn fail(&mut self, message: String, now: Instant) {
        if self.phase == Phase::Released {
            return;
        }
        if self.failure.is_none() {
            self.failure = Some(message);
        }
        self.stop = true;
        self.pending = None;
        if !self.stalled && self.deadline.is_none() {
            self.deadline = Some(now + TIMEOUT);
        }
    }
    fn current(&self, packet: &Packet) -> bool {
        !self.stop
            && !self.target.occluded
            && !self.acquisition_occluded
            && self.target.size == packet.size
            && self.target.generation == packet.stamp.generation
            && self.target.viewport_revision == packet.stamp.viewport_revision
    }
    fn expire(&mut self, now: Instant) {
        if self.deadline.is_some_and(|deadline| now >= deadline) {
            self.fail(format!("Vulkan {:?} exceeded its application deadline; surface ownership is unresolved", self.phase), now);
            self.stalled = true;
            self.deadline = None;
        }
    }
}

pub(super) struct Shared {
    state: Mutex<State>,
    ready: Condvar,
    wake: Arc<dyn Fn() + Send + Sync>,
}
impl Shared {
    pub fn new(target: Target, wake: Arc<dyn Fn() + Send + Sync>, now: Instant) -> Self {
        Self::with_raster(target, wake, now, false)
    }
    pub fn with_raster(
        target: Target,
        wake: Arc<dyn Fn() + Send + Sync>,
        now: Instant,
        native_raster: bool,
    ) -> Self {
        Self {
            state: Mutex::new(State::new(target, now, native_raster)),
            ready: Condvar::new(),
            wake,
        }
    }
    pub fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }
    /// Clear and inspect the wake bit in the SAME critical section on the UI.
    /// Publications after that section must schedule another wake.
    pub fn observe(&self, now: Instant, observe: impl FnOnce(&mut State)) {
        let mut state = self.lock();
        state.expire(now);
        state.wake_pending = false;
        observe(&mut state);
        if state.stop {
            self.ready.notify_one();
        }
    }
    fn notify(&self) {
        let send = {
            let mut state = self.lock();
            !std::mem::replace(&mut state.wake_pending, true)
        };
        // In particular, callback/driver code must never reenter a held mutex.
        if send {
            (self.wake)();
        }
    }
    pub fn fail(&self, error: impl fmt::Display) {
        let message = bounded(error);
        self.lock().fail(message, Instant::now());
        self.ready.notify_one();
        self.notify();
    }
    pub fn request_stop(&self) {
        let mut state = self.lock();
        if state.phase == Phase::Released {
            return;
        }
        state.stop = true;
        state.pending = None;
        if !state.stalled && state.deadline.is_none() {
            state.deadline = Some(Instant::now() + TIMEOUT);
        }
        drop(state);
        self.ready.notify_one();
        self.notify();
    }
    pub fn invalidate(&self, target: Target) {
        let mut state = self.lock();
        if target.generation < state.target.generation
            || target.viewport_revision < state.target.viewport_revision
        {
            state.fail("nonmonotonic presentation target".into(), Instant::now());
        } else {
            if state.target != target {
                state.pending = None;
            }
            // An explicit unoccluded update must resume even if size is equal.
            state.acquisition_occluded = false;
            state.target = target;
            if u64::from(target.size.0) * u64::from(target.size.1) > MAX_PIXELS as u64
                || target.size.0 > 8192
                || target.size.1 > 8192
            {
                state.fail(
                    "Vulkan target exceeds the four-megapixel/8192-axis budget".into(),
                    Instant::now(),
                );
            }
        }
        drop(state);
        self.ready.notify_one();
        self.notify();
    }
    pub fn submit(&self, packet: Packet) -> Submission {
        let mut state = self.lock();
        if state.stop {
            return Submission::Stopped;
        }
        if !state.current(&packet) || packet.stamp.serial <= state.last_serial {
            return Submission::IgnoredStale;
        }
        state.last_serial = packet.stamp.serial;
        let replaced = state.pending.take().is_some(); // Drop old capacity before retaining the new packet.
        state.pending = Some(packet);
        if state
            .live_bytes()
            .is_none_or(|bytes| bytes > APPLICATION_BUDGET)
        {
            state.fail(
                "Vulkan application buffer budget exhausted".into(),
                Instant::now(),
            );
            drop(state);
            self.notify();
            self.ready.notify_one();
            return Submission::Stopped;
        }
        drop(state);
        self.ready.notify_one();
        if replaced {
            Submission::Replaced
        } else {
            Submission::Queued
        }
    }
    pub fn initialized(&self, diagnostic: String) {
        let mut state = self.lock();
        state.expire(Instant::now());
        if !state.stop {
            state.phase = Phase::Idle;
            state.deadline = None;
            state.redraw = true;
            state.diagnostic = Some(diagnostic);
        }
        drop(state);
        self.notify();
    }
    pub fn take(&self) -> Option<Packet> {
        let mut state = self.lock();
        loop {
            if state.stop {
                return None;
            }
            if !state.target.occluded
                && !state.acquisition_occluded
                && state.target.size.0 != 0
                && state.target.size.1 != 0
                && let Some(packet) = state.pending.take()
            {
                if !state.current(&packet) {
                    continue;
                }
                state.active_bytes = packet.bytes();
                state.phase = Phase::Preparing;
                state.deadline = Some(Instant::now() + TIMEOUT);
                drop(state);
                self.notify();
                return Some(packet);
            }
            state = self
                .ready
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
    }
    pub fn current(&self, packet: &Packet) -> bool {
        self.lock().current(packet)
    }
    pub fn stopped(&self) -> bool {
        self.lock().stop
    }
    pub fn deadline(&self) -> Result<Instant, String> {
        let state = self.lock();
        if state.stalled {
            return Err("Vulkan owner stalled".into());
        }
        state
            .deadline
            .ok_or_else(|| "missing active Vulkan deadline".into())
    }
    pub fn phase(&self, phase: Phase) {
        self.lock().phase = phase;
    }
    pub fn reserve_scratch(&self, scratch: usize, readback: usize) -> Result<(), String> {
        let mut state = self.lock();
        state.scratch_bytes = scratch;
        state.readback_bytes = readback;
        if state
            .live_bytes()
            .is_none_or(|bytes| bytes > APPLICATION_BUDGET)
        {
            return Err("Vulkan application buffer budget exhausted".into());
        }
        Ok(())
    }
    #[cfg(feature = "vulkan-raster")]
    pub fn reserve_native(&self, planned: usize, readback: usize) -> Result<(), String> {
        if planned
            .checked_add(readback)
            .is_none_or(|n| n > 24 * 1024 * 1024)
        {
            return Err("native active GPU buffers exceed 24 MiB".into());
        }
        let mut state = self.lock();
        state.active_gpu_bytes = planned;
        state.readback_bytes = readback;
        if state
            .live_bytes()
            .is_none_or(|bytes| bytes > APPLICATION_BUDGET)
        {
            return Err("Vulkan application buffer budget exhausted".into());
        }
        Ok(())
    }
    pub fn occluded(&self) {
        let mut state = self.lock();
        state.acquisition_occluded = true;
        state.pending = None;
    }
    /// Call only after the acquired texture, queued write/submission, packet and
    /// readback have all retired/dropped. Retained conversion capacity stays charged.
    pub fn idle(&self) {
        let mut state = self.lock();
        state.expire(Instant::now());
        state.active_bytes = 0;
        state.readback_bytes = 0;
        state.active_gpu_bytes = 0;
        if !state.stop {
            state.phase = Phase::Idle;
            state.deadline = None;
        }
    }
    pub fn verified(
        &self,
        packet: &Packet,
        route: VerifiedRoute,
        bytes: u64,
    ) -> Result<(), String> {
        let mut state = self.lock();
        #[cfg(feature = "vulkan-raster")]
        let native = route == VerifiedRoute::NativeRaster;
        #[cfg(not(feature = "vulkan-raster"))]
        let native = {
            let _ = route;
            false
        };
        if native != state.native_raster || native != packet.is_native() {
            return Err("Vulkan verification route mismatch".into());
        }
        if bytes != u64::from(packet.size.0) * u64::from(packet.size.1) * 4 {
            return Err("Vulkan verification byte count mismatch".into());
        }
        if packet.stamp.serial <= state.last_verified_serial {
            return Err("duplicate Vulkan verification serial".into());
        }
        state.last_verified_serial = packet.stamp.serial;
        state.verified += 1;
        state.verified_bytes += bytes;
        // No redraw is requested for completion, including verification.
        Ok(())
    }
    #[cfg(feature = "vulkan-raster")]
    pub fn first_native_presented(&self) -> bool {
        !std::mem::replace(&mut self.lock().native_presented, true)
    }
    pub fn begin_release(&self) {
        let mut state = self.lock();
        state.phase = Phase::Releasing;
        state.stop = true;
        state.pending = None;
        if !state.stalled && state.deadline.is_none() {
            state.deadline = Some(Instant::now() + TIMEOUT);
        }
        drop(state);
        self.notify();
    }
    /// Only the outermost owner thread may call this, after all resource drops.
    pub fn released(&self) {
        let mut state = self.lock();
        state.phase = Phase::Released;
        state.deadline = None;
        state.pending = None;
        state.active_bytes = 0;
        state.scratch_bytes = 0;
        state.readback_bytes = 0;
        state.active_gpu_bytes = 0;
        drop(state);
        self.notify();
    }
}

/// Do not allocate an unbounded driver error before truncating it.
pub(super) fn bounded(value: impl fmt::Display) -> String {
    struct Sink(String);
    impl fmt::Write for Sink {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            let left = 1024usize.saturating_sub(self.0.len());
            let mut end = text.len().min(left);
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            self.0.push_str(&text[..end]);
            Ok(())
        }
    }
    let mut sink = Sink(String::new());
    let _ = fmt::write(&mut sink, format_args!("{value}"));
    sink.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    fn target() -> Target {
        Target {
            generation: 1,
            viewport_revision: 1,
            size: (3, 2),
            occluded: false,
        }
    }
    fn packet(serial: u64, target: Target) -> Packet {
        Packet::from_canvas(
            Canvas::new(target.size.0, target.size.1).unwrap(),
            FrameStamp {
                generation: target.generation,
                viewport_revision: target.viewport_revision,
                serial,
            },
        )
        .unwrap()
    }
    fn shared() -> Shared {
        Shared::new(target(), Arc::new(|| {}), Instant::now())
    }
    #[cfg(feature = "vulkan-raster")]
    fn native_packet(serial: u64, reference: bool) -> Packet {
        let plan = eris_raster_core::plan_with_masks_for_profile(
            eris_raster_core::Profile::Native,
            eris_raster_core::Frame::new(3, 2, 0),
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        Packet::from_native(
            plan,
            reference.then(|| Canvas::new(3, 2).unwrap()),
            FrameStamp {
                generation: 1,
                viewport_revision: 1,
                serial,
            },
        )
        .unwrap()
    }
    #[cfg(feature = "vulkan-raster")]
    #[test]
    fn native_packet_moves_plan_and_reference_and_normal_packet_has_no_target() {
        let normal = native_packet(1, false);
        assert!(normal.cpu_pixels().is_none());
        let Payload::Native { plan, reference } = &normal.payload else {
            panic!("native payload")
        };
        assert!(reference.is_none());
        assert_eq!(normal.bytes(), plan.retained_cpu_bytes().unwrap());
        let pixels = plan.parameters().as_ptr();
        let owned = normal;
        let Payload::Native { plan, .. } = &owned.payload else {
            panic!("native payload")
        };
        assert_eq!(plan.parameters().as_ptr(), pixels);
        let with_reference = native_packet(2, true);
        let Payload::Native { plan, reference } = &with_reference.payload else {
            panic!("native payload")
        };
        assert_eq!(
            with_reference.bytes(),
            plan.retained_cpu_bytes().unwrap() + reference.as_ref().unwrap().capacity() * 4
        );
    }
    #[cfg(feature = "vulkan-raster")]
    #[test]
    fn native_packet_refuses_probe_malformed_reference_and_excess_capacity() {
        let probe = eris_raster_core::plan(eris_raster_core::Frame::new(3, 2, 0), &[]).unwrap();
        assert!(Packet::from_native(probe, None, FrameStamp::default()).is_err());
        for mode in 0..3 {
            let Payload::Native { plan, .. } = native_packet(1, false).payload else {
                panic!("native payload")
            };
            let mut canvas = Canvas::new(if mode == 0 { 2 } else { 3 }, 2).unwrap();
            if mode == 1 {
                canvas.pixels.pop();
            }
            if mode == 2 {
                canvas.pixels.reserve(PIXEL_BYTES / 4);
            }
            assert!(Packet::from_native(plan, Some(canvas), FrameStamp::default()).is_err());
        }
    }
    #[cfg(feature = "vulkan-raster")]
    #[test]
    fn heterogeneous_latest_packet_keeps_active_epoch_deadline_and_all_charges() {
        let shared = Shared::with_raster(target(), Arc::new(|| {}), Instant::now(), true);
        shared.initialized("test".into());
        let first = native_packet(1, true);
        let first_bytes = first.bytes();
        assert_eq!(shared.submit(first), Submission::Queued);
        let active = shared.take().unwrap();
        let deadline = shared.lock().deadline;
        shared.reserve_scratch(64, 0).unwrap(); // retained earlier CPU upload Vec
        shared.reserve_native(808, 512).unwrap();
        for serial in 2..=101 {
            let next = if serial % 2 == 0 {
                packet(serial, target())
            } else {
                native_packet(serial, serial % 3 == 0)
            };
            let pending_bytes = next.bytes();
            let expected = if serial == 2 {
                Submission::Queued
            } else {
                Submission::Replaced
            };
            assert_eq!(shared.submit(next), expected);
            let state = shared.lock();
            assert_eq!(state.deadline, deadline);
            assert_eq!(
                state.live_bytes().unwrap(),
                PIXEL_BYTES + first_bytes + pending_bytes + 64 + 808 + 512
            );
            drop(state);
            assert!(shared.current(&active)); // a newer same-epoch serial does not cancel it
        }
        shared.phase(Phase::Encoding);
        shared.phase(Phase::Retiring);
        assert_eq!(shared.lock().deadline, deadline);
        drop(active);
        shared.idle();
        let state = shared.lock();
        assert_eq!(
            (
                state.active_bytes,
                state.active_gpu_bytes,
                state.readback_bytes
            ),
            (0, 0, 0)
        );
        assert_eq!(state.scratch_bytes, 64);
    }
    #[cfg(feature = "vulkan-raster")]
    #[test]
    fn native_readback_and_global_ledger_have_independent_exact_boundaries() {
        let shared = shared();
        // Synthetic allocation ledger boundaries, not large real GPU buffers.
        shared
            .reserve_native(16 * 1024 * 1024, 8 * 1024 * 1024)
            .unwrap();
        assert!(
            shared
                .reserve_native(16 * 1024 * 1024, 8 * 1024 * 1024 + 1)
                .is_err()
        );
        assert!(shared.reserve_native(usize::MAX, 1).is_err());
        let active = 24 * 1024 * 1024;
        let exact_scratch = APPLICATION_BUDGET - PIXEL_BYTES - active;
        shared
            .reserve_scratch(exact_scratch, 8 * 1024 * 1024)
            .unwrap();
        assert_eq!(shared.lock().live_bytes(), Some(APPLICATION_BUDGET));
        assert!(
            shared
                .reserve_scratch(exact_scratch + 1, 8 * 1024 * 1024)
                .is_err()
        );
    }
    #[cfg(feature = "vulkan-raster")]
    #[test]
    fn fallback_upload_cannot_satisfy_native_verification_or_double_count() {
        let shared = Shared::with_raster(target(), Arc::new(|| {}), Instant::now(), true);
        assert!(
            shared
                .verified(&packet(1, target()), VerifiedRoute::CpuUpload, 24)
                .is_err()
        );
        let native = native_packet(2, true);
        assert!(
            shared
                .verified(&native, VerifiedRoute::NativeRaster, 23)
                .is_err()
        );
        assert!(
            shared
                .verified(&native, VerifiedRoute::CpuUpload, 24)
                .is_err()
        );
        assert_eq!(shared.lock().verified, 0);
        shared
            .verified(&native, VerifiedRoute::NativeRaster, 24)
            .unwrap();
        assert_eq!(shared.lock().verified_bytes, 24);
        assert!(
            shared
                .verified(&native, VerifiedRoute::NativeRaster, 24)
                .is_err()
        );
    }
    #[test]
    fn capacity_not_just_dimensions_is_bounded_and_pixels_are_moved() {
        let mut canvas = Canvas::new(3, 2).unwrap();
        let original = canvas.pixels.as_ptr();
        assert_eq!(
            Packet::from_canvas(canvas, FrameStamp::default())
                .unwrap()
                .cpu_pixels()
                .unwrap()
                .as_ptr(),
            original
        );
        canvas = Canvas::new(1, 1).unwrap();
        canvas.pixels.reserve(PIXEL_BYTES);
        assert!(Packet::from_canvas(canvas, FrameStamp::default()).is_err());
        canvas = Canvas::new(1, 1).unwrap();
        canvas.width = 8193;
        assert!(Packet::from_canvas(canvas, FrameStamp::default()).is_err());
    }
    #[test]
    fn ten_thousand_updates_retain_one_latest_packet_without_resetting_deadline() {
        let shared = shared();
        let deadline = shared.lock().deadline;
        let mut target = target();
        for serial in 1..=10_000 {
            target.generation += 1;
            target.viewport_revision += 1;
            target.size = if serial % 2 == 0 { (3, 2) } else { (2, 3) };
            shared.invalidate(target);
            assert_eq!(shared.submit(packet(serial, target)), Submission::Queued);
            assert!(shared.lock().live_bytes().unwrap() <= PIXEL_BYTES + 24);
        }
        assert_eq!(shared.lock().deadline, deadline);
        assert_eq!(shared.lock().pending.as_ref().unwrap().stamp.serial, 10_000);
        assert_eq!(shared.submit(packet(10_001, target)), Submission::Replaced);
        assert_eq!(
            shared.submit(packet(10_000, target)),
            Submission::IgnoredStale
        );
        target.viewport_revision += 1;
        shared.invalidate(target);
        assert!(shared.lock().pending.is_none());
    }
    #[test]
    fn stale_active_frames_and_occlusion_do_not_resume_on_new_packets() {
        let shared = shared();
        shared.initialized("test".into());
        shared.submit(packet(1, target()));
        let active = shared.take().unwrap();
        let original_deadline = shared.lock().deadline;
        let mut changed = target();
        changed.viewport_revision += 1;
        changed.size = (2, 3);
        shared.invalidate(changed);
        changed.viewport_revision += 1;
        changed.size = target().size;
        shared.invalidate(changed);
        assert!(!shared.current(&active));
        for phase in [
            Phase::Acquired,
            Phase::UploadQueued,
            Phase::Submitted,
            Phase::Retiring,
        ] {
            shared.phase(phase);
            assert_eq!(shared.lock().deadline, original_deadline);
        }
        drop(active);
        shared.idle();
        shared.occluded();
        assert_eq!(shared.submit(packet(2, changed)), Submission::IgnoredStale);
        shared.invalidate(changed);
        assert_eq!(shared.submit(packet(3, changed)), Submission::Queued);
        let active = shared.take().unwrap();
        assert!(shared.current(&active));
        changed.occluded = true;
        shared.invalidate(changed);
        assert!(!shared.current(&active));
    }
    #[test]
    fn expiry_and_stop_never_acknowledge_release_or_reset_on_traffic() {
        let shared = shared();
        let deadline = shared.lock().deadline.unwrap();
        shared.observe(deadline, |s| {
            assert!(s.stalled && s.stop);
            assert_eq!(s.phase, Phase::Initializing);
        });
        shared.initialized("late success".into());
        shared.invalidate(target());
        shared.begin_release();
        assert_eq!(shared.lock().phase, Phase::Releasing);
        assert!(shared.lock().deadline.is_none());
        assert!(shared.lock().diagnostic.is_none());
        shared.released();
        assert_eq!(shared.lock().phase, Phase::Released);
        assert!(shared.lock().failure.is_some());
    }
    #[test]
    fn coalesced_wake_is_drained_with_state_and_allows_reentrancy() {
        let count = Arc::new(AtomicUsize::new(0));
        let copy = count.clone();
        let slot: Arc<Mutex<Option<std::sync::Weak<Shared>>>> = Arc::new(Mutex::new(None));
        let copy_slot = slot.clone();
        let shared = Arc::new(Shared::new(
            target(),
            Arc::new(move || {
                copy.fetch_add(1, Ordering::Relaxed);
                let owner = copy_slot
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap();
                assert!(owner.lock().wake_pending); // Proves wake executes outside lock.
            }),
            Instant::now(),
        ));
        *slot.lock().unwrap() = Some(Arc::downgrade(&shared));
        shared.initialized("test".into());
        shared.invalidate(target());
        assert_eq!(count.load(Ordering::Relaxed), 1);
        shared.observe(Instant::now(), |s| {
            s.redraw = false;
        });
        shared.submit(packet(1, target()));
        let active = shared.take().unwrap();
        assert_eq!(count.load(Ordering::Relaxed), 2);
        shared.observe(Instant::now(), |s| assert!(!s.redraw));
        shared
            .verified(&active, VerifiedRoute::CpuUpload, 24)
            .unwrap();
        drop(active);
        shared.idle();
        assert_eq!(count.load(Ordering::Relaxed), 2);
        assert!(shared.lock().deadline.is_none());
        shared.fail("callback error");
        assert_eq!(count.load(Ordering::Relaxed), 3);
    }
    #[test]
    fn bounded_errors_and_resource_ledger_are_checked() {
        let message = "🦀".repeat(4096);
        assert_eq!(bounded(message).len(), 1024);
        let shared = shared();
        shared.submit(packet(1, target()));
        let active = shared.take().unwrap();
        shared
            .reserve_scratch(PIXEL_BYTES, 18 * 1024 * 1024)
            .unwrap();
        assert!(shared.reserve_scratch(usize::MAX, 1).is_err());
        shared.reserve_scratch(0, 0).unwrap();
        shared
            .verified(&active, VerifiedRoute::CpuUpload, 24)
            .unwrap();
        assert!(
            shared
                .verified(&active, VerifiedRoute::CpuUpload, 24)
                .is_err()
        );
    }
}
