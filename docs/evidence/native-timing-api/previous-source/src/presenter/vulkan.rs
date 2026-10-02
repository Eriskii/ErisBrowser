//! Optional native Vulkan presentation with whole-frame CPU/native routes.
//! All graphics objects live on one owner thread; the UI only shares CPU state.
mod control;
#[cfg(feature = "vulkan-raster")]
mod native;
mod transfer;
use super::timing::{
    TimingReport, TimingRequest, TimingRoute, TimingSubmission,
    record::{self, Step, Trace},
};
use super::{FrameStamp, Notice, Submission, Target};
use control::{Packet, Phase, Shared, VerifiedRoute, bounded};
use eris::graphics::Canvas;
use std::{
    future::Future,
    pin::pin,
    sync::{Arc, mpsc},
    task::{Context, Poll, Wake, Waker},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use winit::window::Window;

type Result<T> = std::result::Result<T, String>;

pub(super) struct Worker {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
    verify_frames: u8,
    native_raster: bool,
    benchmark_frames: u8,
    #[cfg(feature = "vulkan-raster")]
    last_fallback: Option<String>,
    failure_reported: bool,
    release_reported: bool,
}
impl Worker {
    pub fn new(
        window: Arc<Window>,
        target: Target,
        verify_frames: u8,
        native_raster: bool,
        benchmark_frames: u8,
        benchmark_check: bool,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<Self> {
        let shared = Arc::new(if native_raster {
            Shared::with_raster(target, wake, Instant::now(), true)
        } else {
            Shared::new(target, wake, Instant::now())
        });
        shared.enable_benchmark(benchmark_frames, benchmark_check)?;
        let owner = shared.clone();
        let thread = thread::Builder::new()
            .name("eris-vulkan-presenter".into())
            .spawn(move || {
                // Returning from this function means ALL API objects/locals have
                // dropped. Panic, timeout, destroy request or a finished join handle
                // alone never reaches the release acknowledgment below.
                let result = run_owner(
                    window,
                    &owner,
                    verify_frames,
                    native_raster,
                    benchmark_frames,
                );
                if let Err(error) = result {
                    owner.fail(error);
                }
                owner.released();
            })
            .map_err(|e| bounded(format_args!("Vulkan thread spawn: {e}")))?;
        Ok(Self {
            shared,
            thread: Some(thread),
            verify_frames,
            native_raster,
            benchmark_frames,
            #[cfg(feature = "vulkan-raster")]
            last_fallback: None,
            failure_reported: false,
            release_reported: false,
        })
    }
    pub fn submit(
        &self,
        canvas: Canvas,
        stamp: FrameStamp,
        timing: Option<TimingRequest>,
    ) -> Result<Submission> {
        match Packet::from_canvas(canvas, stamp) {
            Ok(mut packet) => {
                if let Some(timing) = timing {
                    packet.attach_timing(timing)?;
                }
                Ok(self.shared.submit(packet))
            }
            Err(error) => {
                self.shared.fail(error);
                Ok(Submission::Stopped)
            }
        }
    }
    #[cfg(feature = "vulkan-raster")]
    pub fn wants_native(&self) -> bool {
        self.native_raster && !self.shared.stopped()
    }
    #[cfg(feature = "vulkan-raster")]
    pub fn needs_native_reference(&self) -> bool {
        self.wants_native() && self.shared.lock().verified < self.verify_frames
    }
    #[cfg(feature = "vulkan-raster")]
    pub fn submit_native(
        &self,
        plan: eris_raster_core::Plan,
        reference: Option<Canvas>,
        stamp: FrameStamp,
        timing: Option<TimingRequest>,
    ) -> Result<Submission> {
        let mut packet = Packet::from_native(plan, reference, stamp)?;
        if let Some(timing) = timing {
            packet.attach_timing(timing)?;
        }
        Ok(self.shared.submit(packet))
    }
    pub fn benchmark_frames(&self) -> u8 {
        self.benchmark_frames
    }
    pub fn timing_ready(&self) -> bool {
        self.shared.timing_ready()
    }
    pub fn take_timing_report(&mut self) -> Result<TimingReport> {
        self.shared.take_timing_report()
    }
    pub fn arm_benchmark_check(&mut self) -> Result<()> {
        self.shared.arm_benchmark_check()
    }
    pub fn benchmark_check_complete(&self) -> bool {
        self.shared.benchmark_check_complete()
    }
    #[cfg(feature = "vulkan-raster")]
    pub fn note_native_fallback(&mut self, reason: &str) {
        if !self.native_raster {
            return;
        }
        let reason = bounded(reason);
        if self.last_fallback.as_ref() != Some(&reason) {
            eprintln!(
                "presenter: native raster admission fallback; complete CPU upload; reason={reason}"
            );
            self.last_fallback = Some(reason);
        }
    }
    pub fn invalidate(&self, target: Target) {
        self.shared.invalidate(target);
    }
    pub fn verification_requested(&self) -> u8 {
        self.verify_frames
    }
    pub fn request_stop(&self) {
        self.shared.request_stop();
    }
    pub fn shutdown_complete(&self) -> bool {
        let state = self.shared.lock();
        state.phase == Phase::Released || state.stalled
    }
    pub fn benchmark_owner_released(&self) -> bool {
        self.shared.lock().phase == Phase::Released
    }
    pub fn service(&mut self, now: Instant) -> (Notice, bool) {
        if self.thread.as_ref().is_some_and(JoinHandle::is_finished)
            && let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            self.shared
                .fail("Vulkan owner panicked; resource release is unconfirmed");
        }
        let mut notice = Notice::default();
        let mut released = false;
        let mut check_timing_release = false;
        self.shared.observe(now, |state| {
            notice.redraw = std::mem::take(&mut state.redraw);
            notice.message = state.diagnostic.take();
            notice.timing_completion = state.timing_completion.take();
            if !self.failure_reported
                && let Some(error) = &state.failure
            {
                self.failure_reported = true;
                notice.message = Some(format!("presenter: {error}"));
                if self.verify_frames > 0 || self.benchmark_frames > 0 {
                    notice.error = Some(error.clone());
                }
            }
            released = state.phase == Phase::Released;
            if released && !self.release_reported {
                self.release_reported = true;
                check_timing_release = self.benchmark_frames != 0;
                if self.verify_frames > 0
                    && state.verified < self.verify_frames
                    && notice.error.is_none()
                {
                    notice.error = Some(format!(
                        "Vulkan verification incomplete: {}/{} frames",
                        state.verified, self.verify_frames
                    ));
                }
            }
        });
        if check_timing_release && notice.error.is_none() {
            notice.error = self.shared.timing_complete().err();
        }
        (notice, released)
    }
    pub fn next_deadline(&self) -> Option<Instant> {
        self.shared.lock().deadline
    }
    pub fn finish(&mut self) -> Result<()> {
        self.request_stop();
        if self.benchmark_frames > 0 {
            return self.shared.timing_complete();
        }
        let state = self.shared.lock();
        if self.verify_frames == 0 {
            return Ok(());
        }
        if let Some(error) = &state.failure {
            return Err(error.clone());
        }
        if state.verified < self.verify_frames {
            return Err(format!(
                "Vulkan verification incomplete: {}/{} distinct frames",
                state.verified, self.verify_frames
            ));
        }
        eprintln!(
            "presenter: Vulkan acquired-texture verification passed; route={} frames={} compared_bytes={} last_serial={} (before compositor)",
            if self.native_raster {
                "native-raster"
            } else {
                "cpu-upload"
            },
            state.verified,
            state.verified_bytes,
            state.last_verified_serial
        );
        Ok(())
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.shared.request_stop();
        if self.thread.as_ref().is_some_and(JoinHandle::is_finished)
            && let Some(thread) = self.thread.take()
        {
            let _ = thread.join();
        }
        // An unfinished driver call cannot be joined on the UI thread. The
        // remaining owner retains the window; no competing surface is created.
    }
}

struct ThreadWake(thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}
fn remaining(deadline: Instant) -> Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or_else(|| "Vulkan operation exceeded its application deadline".into())
}
fn wait_future<F: Future>(future: F, deadline: Instant) -> Result<F::Output> {
    let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        remaining(deadline)?;
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => {
                remaining(deadline)?;
                return Ok(value);
            }
            Poll::Pending => {
                thread::park_timeout(remaining(deadline)?.min(Duration::from_millis(10)))
            }
        }
    }
}

struct Gpu {
    // Explicit owned fields are dropped before outer run_owner acknowledges
    // release. No callback retains this object or another graphics API handle.
    surface: wgpu::Surface<'static>,
    queue: wgpu::Queue,
    device: wgpu::Device,
    _instance: wgpu::Instance,
    window: Arc<Window>,
    config: wgpu::SurfaceConfiguration,
    configured: bool,
    converted: Vec<u8>,
    #[cfg(feature = "vulkan-raster")]
    native: Option<native::Kernels>,
}
impl Gpu {
    fn new(
        window: Arc<Window>,
        shared: &Arc<Shared>,
        verify: bool,
        native_raster: bool,
    ) -> Result<(Self, String)> {
        let deadline = shared.deadline()?;
        remaining(deadline)?;
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(descriptor);
        let surface = instance.create_surface(window.clone()).map_err(bounded)?;
        let adapters = wait_future(
            instance.enumerate_adapters(wgpu::Backends::VULKAN),
            deadline,
        )?;
        // This bounds our inspection, not the driver's enumeration allocation.
        if adapters.is_empty() || adapters.len() > 16 {
            return Err("Vulkan adapter inventory is empty or exceeds 16".into());
        }
        let usage = wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | if verify {
                wgpu::TextureUsages::COPY_SRC
            } else {
                wgpu::TextureUsages::empty()
            };
        let mut selected = None;
        for adapter in adapters {
            remaining(deadline)?;
            let info = adapter.get_info();
            if info.backend != wgpu::Backend::Vulkan || !adapter.is_surface_supported(&surface) {
                continue;
            }
            let caps = surface.get_capabilities(&adapter);
            if !caps.usages.contains(usage)
                || !caps.present_modes.contains(&wgpu::PresentMode::Fifo)
                || !caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque)
            {
                continue;
            }
            let Some(format) = [
                wgpu::TextureFormat::Bgra8Unorm,
                wgpu::TextureFormat::Rgba8Unorm,
            ]
            .into_iter()
            .find(|f| {
                caps.formats.contains(f)
                    && caps
                        .color_spaces(*f)
                        .contains(wgpu::SurfaceColorSpaces::SRGB)
            }) else {
                continue;
            };
            let priority = match info.device_type {
                wgpu::DeviceType::DiscreteGpu => 0,
                wgpu::DeviceType::IntegratedGpu => 1,
                wgpu::DeviceType::VirtualGpu => 2,
                wgpu::DeviceType::Other => 3,
                wgpu::DeviceType::Cpu => 4,
            };
            if selected
                .as_ref()
                .is_none_or(|(rank, _, _, _)| priority < *rank)
            {
                selected = Some((priority, adapter, format, info));
            }
        }
        let (_, adapter, format, info) = selected.ok_or(
            "no Vulkan adapter supports the required opaque FIFO copy surface/verification usage",
        )?;
        let available = adapter.limits();
        let limits = wgpu::Limits {
            max_texture_dimension_2d: 8192.min(available.max_texture_dimension_2d),
            ..wgpu::Limits::downlevel_defaults()
        };
        let (device, queue) = wait_future(
            adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("Eris Vulkan presentation"),
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            }),
            deadline,
        )?
        .map_err(bounded)?;
        let errors = shared.clone();
        device.on_uncaptured_error(Arc::new(move |error| errors.fail(error)));
        let lost = shared.clone();
        device.set_device_lost_callback(move |reason, message| {
            // Explicit device destruction during release is not an unexpected loss.
            if reason != wgpu::DeviceLostReason::Destroyed || lost.lock().phase != Phase::Releasing
            {
                lost.fail(format_args!("Vulkan device lost ({reason:?}): {message}"));
            }
        });
        #[cfg(feature = "vulkan-raster")]
        let native = if native_raster {
            Some(native::Kernels::new(&device, deadline)?)
        } else {
            None
        };
        let diagnostic = bounded(format_args!(
            "presenter: Vulkan adapter={:?} type={:?} driver={:?} format={format:?} color_space=Srgb alpha=Opaque mode=Fifo; raster={}",
            info.name,
            info.device_type,
            info.driver,
            if native_raster {
                "native-shaders with complete CPU admission fallback"
            } else {
                "CPU upload"
            }
        ));
        remaining(deadline)?;
        let gpu = Self {
            surface,
            queue,
            device,
            _instance: instance,
            window,
            config: wgpu::SurfaceConfiguration {
                usage,
                format,
                color_space: wgpu::SurfaceColorSpace::Srgb,
                width: 1,
                height: 1,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
            },
            configured: false,
            converted: Vec::new(),
            #[cfg(feature = "vulkan-raster")]
            native,
        };
        Ok((gpu, diagnostic))
    }
    fn configure(&mut self, packet: &Packet, shared: &Shared) -> Result<()> {
        remaining(shared.deadline()?)?;
        let limits = self.device.limits();
        if packet.size.0 > limits.max_texture_dimension_2d
            || packet.size.1 > limits.max_texture_dimension_2d
        {
            return Err("Vulkan target exceeds effective device texture limits".into());
        }
        self.config.width = packet.size.0;
        self.config.height = packet.size.1;
        self.surface.configure(&self.device, &self.config);
        self.configured = true;
        remaining(shared.deadline()?)?;
        Ok(())
    }
    fn acquire(
        &mut self,
        packet: &Packet,
        shared: &Shared,
        deadline: Instant,
        timing: &mut Option<Trace>,
    ) -> Result<Option<(wgpu::SurfaceTexture, bool)>> {
        remaining(deadline)?;
        if !shared.current(packet) {
            return Ok(None);
        }
        if !self.configured || (self.config.width, self.config.height) != packet.size {
            if let Some(trace) = timing {
                trace.configured();
            }
            self.configure(packet, shared)?;
        }
        if !shared.current(packet) {
            return Ok(None);
        }
        let acquired = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Outdated => {
                // No texture was acquired. Exactly one configure retry.
                if !shared.current(packet) {
                    return Ok(None);
                }
                if let Some(trace) = timing {
                    trace.configured();
                }
                self.configure(packet, shared)?;
                self.surface.get_current_texture()
            }
            other => other,
        };
        remaining(deadline)?;
        let (frame, suboptimal) = match acquired {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Occluded => {
                shared.occluded();
                return Ok(None);
            }
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(None),
            other => return Err(bounded(format_args!("Vulkan acquisition: {other:?}"))),
        };
        shared.phase(Phase::Acquired);
        remaining(deadline)?;
        if !shared.current(packet) {
            return Ok(None);
        }
        Ok(Some((frame, suboptimal)))
    }
    fn frame(
        &mut self,
        packet: &Packet,
        shared: &Shared,
        verify: bool,
        timing: &mut Option<Trace>,
    ) -> Result<()> {
        #[cfg(feature = "vulkan-raster")]
        if packet.is_native() {
            return self.frame_native(packet, shared, verify, timing);
        }
        self.frame_cpu(packet, shared, verify, timing)
    }
    fn frame_cpu(
        &mut self,
        packet: &Packet,
        shared: &Shared,
        verify: bool,
        timing: &mut Option<Trace>,
    ) -> Result<()> {
        let pixels = packet.cpu_pixels().ok_or("CPU upload requires pixels")?;
        let deadline = shared.deadline()?;
        record::begin(timing, Step::Acquire);
        let acquired = self.acquire(packet, shared, deadline, timing);
        record::end(timing);
        let Some((frame, suboptimal)) = acquired? else {
            return Ok(());
        };
        record::begin(timing, Step::EncodeUpload);
        let (packed, padded, readback_bytes) = transfer::row_layout(packet.size)?;
        // During growth, the allocator may retain both old and new conversion
        // storage briefly. Reserve that peak before allocating the new bytes.
        shared.reserve_scratch(
            self.converted
                .capacity()
                .checked_add(pixels.len() * 4)
                .ok_or("Vulkan conversion capacity overflow")?,
            if verify { readback_bytes } else { 0 },
        )?;
        transfer::convert(pixels, self.config.format, &mut self.converted)?;
        shared.reserve_scratch(
            self.converted.capacity(),
            if verify { readback_bytes } else { 0 },
        )?;
        let readback = if verify {
            if readback_bytes as u64 > self.device.limits().max_buffer_size {
                return Err("Vulkan readback exceeds effective device buffer limits".into());
            }
            Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Eris acquired surface verification"),
                size: readback_bytes as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }))
        } else {
            None
        };
        if !shared.current(packet) {
            return Ok(());
        }
        remaining(deadline)?;
        let extent = wgpu::Extent3d {
            width: packet.size.0,
            height: packet.size.1,
            depth_or_array_layers: 1,
        };
        self.queue.write_texture(
            frame.texture.as_image_copy(),
            &self.converted,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(packed),
                rows_per_image: Some(packet.size.1),
            },
            extent,
        );
        record::end(timing);
        shared.phase(Phase::UploadQueued);
        // Once write_texture returns, staging exists. Even if the epoch changed
        // in that call, submit it ONCE and retire it before any next upload.
        record::begin(timing, Step::Submit);
        let submission = if let Some(buffer) = &readback {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Eris surface readback"),
                });
            encoder.copy_texture_to_buffer(
                frame.texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded),
                        rows_per_image: Some(packet.size.1),
                    },
                },
                extent,
            );
            self.queue.submit([encoder.finish()])
        } else {
            self.queue.submit([])
        };
        record::end(timing);
        if let Some(trace) = timing {
            trace.submitted(TimingSubmission::Draws);
        }
        shared.phase(Phase::Submitted);
        let mapping = readback.as_ref().map(|buffer| {
            let (tx, rx) = mpsc::sync_channel(1);
            buffer.map_async(wgpu::MapMode::Read, .., move |result| {
                let _ = tx.try_send(result);
            });
            rx
        });
        shared.phase(Phase::Retiring);
        record::begin(timing, Step::CompletionWait);
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(remaining(deadline)?),
            })
            .map_err(bounded)?;
        record::end(timing);
        remaining(deadline)?;
        if let (Some(buffer), Some(mapping)) = (&readback, mapping) {
            mapping
                .recv_timeout(remaining(deadline)?)
                .map_err(bounded)?
                .map_err(bounded)?;
            let mapped = buffer.get_mapped_range(..).map_err(bounded)?;
            let compared = transfer::compare(&mapped, &self.converted, packet.size, padded);
            drop(mapped);
            buffer.unmap();
            compared?;
        }
        // Callbacks may have failed without the graphics call returning Err.
        if shared.current(packet) {
            record::begin(timing, Step::Present);
            self.window.pre_present_notify();
            self.queue.present(frame);
            record::end(timing);
            remaining(deadline)?;
            if shared.stopped() {
                return Err("Vulkan failed during present".into());
            }
            if let Some(trace) = timing {
                trace.presented();
            }
            if verify {
                shared.verified(packet, VerifiedRoute::CpuUpload, (pixels.len() * 4) as u64)?;
                eprintln!(
                    "presenter: Vulkan verified serial={} generation={} viewport_revision={} size={}x{} compared_bytes={} exact=true (before compositor)",
                    packet.stamp.serial,
                    packet.stamp.generation,
                    packet.stamp.viewport_revision,
                    packet.size.0,
                    packet.size.1,
                    pixels.len() * 4
                );
            }
        } else {
            drop(frame);
        }
        // The acquired texture has been consumed/dropped before the next configure.
        if suboptimal {
            self.configured = false;
        }
        drop(readback);
        Ok(())
    }
}

fn run_owner(
    window: Arc<Window>,
    shared: &Arc<Shared>,
    verify_frames: u8,
    native_raster: bool,
    benchmark_frames: u8,
) -> Result<()> {
    let init_start = (benchmark_frames != 0).then(Instant::now);
    let result = Gpu::new(window, shared, verify_frames > 0, native_raster);
    let initialization = init_start.map(record::initialization).transpose();
    let (mut gpu, diagnostic) = match result {
        Ok(value) => value,
        Err(error) => {
            shared.begin_release();
            return Err(error);
        }
    };
    match initialization {
        Ok(Some(initialization)) => shared.timing_initialization(initialization),
        Ok(None) => {}
        Err(error) => {
            shared.begin_release();
            drop(gpu);
            return Err(error);
        }
    }
    shared.initialized(diagnostic);
    let result = (|| {
        while let Some(packet) = shared.take() {
            let verify = shared.should_verify(&packet, verify_frames, native_raster);
            let mut timing = packet.timing.map(|tag| {
                Trace::new(
                    tag,
                    packet.stamp,
                    packet.size,
                    if packet.is_native() {
                        TimingRoute::NativeRaster
                    } else {
                        TimingRoute::CpuUpload
                    },
                )
            });
            let result = gpu.frame(&packet, shared, verify, &mut timing);
            drop(packet);
            if let Err(error) = result {
                // Partial timing cannot bypass failed GPU retirement/release or
                // replace its first error. Incomplete telemetry also fails exit.
                if let Some(trace) = timing {
                    let _ = trace
                        .finish(true)
                        .and_then(|sample| shared.timing_failure(sample));
                }
                return Err(error);
            }
            shared.idle();
            if let Some(trace) = timing {
                let sample = trace.finish(false)?;
                if let Err(error) = shared.complete_timing(sample) {
                    // idle may have observed a deadline/device failure after
                    // present. Keep the reached trace, but issue no completion.
                    let _ = shared.timing_failure(super::timing::TimingSample {
                        outcome: super::timing::TimingOutcome::Failed,
                        ..sample
                    });
                    return Err(error);
                }
            }
        }
        Ok(())
    })();
    if let Err(error) = &result {
        shared.fail(error);
    }
    shared.begin_release();
    drop(gpu); // May itself stall. Released is published only by the caller.
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fake_worker(verify_frames: u8) -> Worker {
        Worker {
            shared: Arc::new(Shared::new(
                Target {
                    generation: 1,
                    viewport_revision: 1,
                    size: (3, 2),
                    occluded: false,
                },
                Arc::new(|| {}),
                Instant::now(),
            )),
            thread: None,
            verify_frames,
            native_raster: false,
            benchmark_frames: 0,
            #[cfg(feature = "vulkan-raster")]
            last_fallback: None,
            failure_reported: false,
            release_reported: false,
        }
    }
    #[test]
    fn blocked_drop_does_not_allow_fallback_and_late_real_release_does() {
        struct SlowDrop {
            entered: mpsc::Sender<()>,
            release: mpsc::Receiver<()>,
        }
        impl Drop for SlowDrop {
            fn drop(&mut self) {
                self.entered.send(()).unwrap();
                self.release.recv_timeout(Duration::from_secs(5)).unwrap();
            }
        }
        let mut worker = fake_worker(0);
        let owner = worker.shared.clone();
        let (entered, wait) = mpsc::channel();
        let (release, proceed) = mpsc::channel();
        let (done, completed) = mpsc::channel();
        worker.thread = Some(thread::spawn(move || {
            let resource = SlowDrop {
                entered,
                release: proceed,
            };
            owner.begin_release();
            drop(resource);
            owner.released();
            done.send(()).unwrap();
        }));
        wait.recv_timeout(Duration::from_secs(2)).unwrap();
        let deadline = worker.next_deadline().unwrap();
        assert!(!worker.service(deadline).1);
        assert!(!worker.benchmark_owner_released());
        assert!(worker.shared.lock().stalled);
        assert!(worker.next_deadline().is_none());
        assert!(!worker.service(deadline + Duration::from_secs(10)).1);
        release.send(()).unwrap();
        completed.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(worker.service(Instant::now()).1);
        assert!(worker.benchmark_owner_released());
    }
    #[test]
    fn panicked_thread_without_release_is_not_fallback_authority() {
        let mut worker = fake_worker(0);
        worker.thread = Some(thread::spawn(|| panic!("injected owner failure")));
        let limit = Instant::now() + Duration::from_secs(2);
        while !worker.thread.as_ref().unwrap().is_finished() {
            assert!(Instant::now() < limit);
            thread::sleep(Duration::from_millis(1));
        }
        let (notice, released) = worker.service(Instant::now());
        assert!(!released);
        assert!(notice.message.unwrap().contains("release is unconfirmed"));
        assert!(worker.thread.is_none());
        assert!(!worker.benchmark_owner_released());
        let deadline = worker.next_deadline().unwrap();
        assert!(!worker.service(deadline).1);
        assert!(!worker.benchmark_owner_released());
        assert!(worker.shared.lock().stalled);
    }
    #[test]
    fn verification_fails_for_insufficient_samples_and_any_late_failure() {
        let mut worker = fake_worker(2);
        assert!(worker.finish().unwrap_err().contains("0/2"));
        let mut worker = fake_worker(1);
        let packet = Packet::from_canvas(
            Canvas::new(3, 2).unwrap(),
            FrameStamp {
                generation: 1,
                viewport_revision: 1,
                serial: 1,
            },
        )
        .unwrap();
        worker
            .shared
            .verified(&packet, VerifiedRoute::CpuUpload, 24)
            .unwrap();
        worker.shared.fail("injected late device loss");
        assert!(worker.finish().unwrap_err().contains("late device loss"));
        let (notice, released) = worker.service(Instant::now());
        assert!(!released);
        assert!(notice.error.unwrap().contains("late device loss"));
    }
}
