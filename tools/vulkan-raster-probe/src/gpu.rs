//! Shared bounded Vulkan execution for the standalone probes.
//! Plans and independent targets must be complete before this module is called.
use crate::{MAX_GPU_BUFFER_BYTES, Plan, Result};
use eris_raster_core::gpu::Rasterizer;
use std::{
    future::Future,
    pin::pin,
    sync::{Arc, mpsc},
    task::{Context, Poll, Wake, Waker},
    thread,
    time::{Duration, Instant},
};
const API_TIMEOUT: Duration = Duration::from_secs(5);
const RUN_TIMEOUT: Duration = Duration::from_secs(20);
struct ThreadWake(thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}
pub(crate) struct Deadline(pub(crate) Instant);
impl Deadline {
    pub(crate) fn remaining(&self) -> Result<Duration> {
        self.0
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .map(|d| d.min(API_TIMEOUT))
            .ok_or_else(|| "whole application deadline".into())
    }
    pub(crate) fn wait<F: Future>(&self, future: F) -> Result<F::Output> {
        let until = Instant::now() + self.remaining()?;
        let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
        let mut context = Context::from_waker(&waker);
        let mut future = pin!(future);
        loop {
            self.remaining()?;
            match future.as_mut().poll(&mut context) {
                Poll::Ready(v) => return Ok(v),
                Poll::Pending => {
                    let remaining = until
                        .checked_duration_since(Instant::now())
                        .ok_or("future pending deadline")?;
                    thread::park_timeout(remaining.min(Duration::from_millis(10)));
                }
            }
        }
    }
}

fn raster(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    rasterizer: &Rasterizer,
    p: &Plan,
    expected: &[u32],
    deadline: &Deadline,
) -> Result<()> {
    deadline.remaining()?;
    let frame = p.frame();
    let pixel_bytes = u64::from(frame.width) * u64::from(frame.height) * 4;
    if expected.len() as u64 * 4 != pixel_bytes {
        return Err("fixture length mismatch".into());
    }
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ordered custom compute rasterization"),
    });
    let encoded = rasterizer.encode(device, queue, p, &mut encoder, || {
        deadline.remaining().map(|_| ())
    })?;
    // Core owns one target; the probe owns the separately charged readback.
    // Preserve the planner's two-target cap instead of silently relaxing it.
    if encoded.owned_buffer_bytes().checked_add(pixel_bytes) != Some(p.gpu_buffer_bytes())
        || encoded.planned_buffer_bytes() != p.gpu_buffer_bytes()
        || encoded.output_bytes() != pixel_bytes
        || p.gpu_buffer_bytes() > MAX_GPU_BUFFER_BYTES
    {
        return Err("explicit GPU allocation budget mismatch".into());
    }
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("bounded readback"),
        size: pixel_bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(encoded.output_buffer(), 0, &readback, 0, pixel_bytes);
    let submission = queue.submit([encoder.finish()]);
    let (tx, rx) = mpsc::sync_channel(1);
    readback.map_async(wgpu::MapMode::Read, .., move |result| {
        let _ = tx.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(deadline.remaining()?),
        })
        .map_err(|e| format!("GPU poll: {e}"))?;
    rx.recv_timeout(deadline.remaining()?)
        .map_err(|e| format!("mapping callback: {e}"))?
        .map_err(|e| format!("mapping: {e}"))?;
    let comparison = {
        let mapped = readback
            .get_mapped_range(..)
            .map_err(|e| format!("mapped range: {e}"))?;
        let (pixels, remainder) = mapped.as_chunks::<4>();
        if !remainder.is_empty() || pixels.len() != expected.len() {
            Err("readback size mismatch".into())
        } else {
            pixels
                .iter()
                .zip(expected)
                .enumerate()
                .find_map(|(i, (bytes, expected))| {
                    let actual = u32::from_le_bytes(*bytes);
                    (actual != *expected).then(|| {
                        format!(
                            "pixel ({},{}) actual={actual:08x} expected={expected:08x}",
                            i % frame.width as usize,
                            i / frame.width as usize
                        )
                    })
                })
                .map_or(Ok(()), Err)
        }
    };
    readback.unmap();
    encoded.destroy_after_completion();
    readback.destroy();
    comparison
}
/// Enumerate Vulkan adapters and optionally compare every supplied plan.
/// The callback is synchronous and must perform only bounded output work.
/// Driver, mapping, comparison and callback failures are never CPU fallback.
pub fn run_plans(
    selected: Option<usize>,
    plans: &[(&str, &[u32], &Plan)],
    mut on_pass: impl FnMut(&str, &Plan, &[u32]) -> Result<()>,
) -> Result<()> {
    if plans.len() > 30 || (selected.is_some() && plans.is_empty()) {
        return Err("GPU plan count outside 1..=30".into());
    }
    let deadline = Deadline(Instant::now() + RUN_TIMEOUT);
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(descriptor);
    if wgpu::Instance::enabled_backend_features() != wgpu::Backends::VULKAN {
        return Err("unexpected compiled backend".into());
    }
    let adapters = deadline.wait(instance.enumerate_adapters(wgpu::Backends::VULKAN))?;
    if adapters.is_empty() || adapters.len() > 16 {
        return Err("adapter count outside 1..=16".into());
    }
    for (i, adapter) in adapters.iter().enumerate() {
        let info = adapter.get_info();
        if info.backend != wgpu::Backend::Vulkan {
            return Err("non-Vulkan adapter".into());
        }
        println!(
            "ADAPTER {i} vendor={:#x} device={:#x} type={:?} backend={:?} name={:?} driver={:?} info={:?}",
            info.vendor,
            info.device,
            info.device_type,
            info.backend,
            info.name,
            info.driver,
            info.driver_info
        );
    }
    let Some(index) = selected else {
        return Ok(());
    };
    let adapter = adapters.get(index).ok_or("adapter outside inventory")?;
    let (device, queue) = deadline
        .wait(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("bounded custom rectangle and image blending experiment"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }))?
        .map_err(|e| format!("device: {e}"))?;
    deadline.remaining()?;
    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let oom = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    let rasterizer = Rasterizer::new(&device, plans.iter().any(|(_, _, plan)| plan.has_glyphs()));
    let outcome: Result<()> = (|| {
        for &(name, expected, plan) in plans {
            raster(&device, &queue, &rasterizer, plan, expected, &deadline)
                .map_err(|e| format!("{name}: {e}"))?;
            on_pass(name, plan, expected)?;
        }
        Ok(())
    })();
    for scope in [internal, oom, validation] {
        if let Some(error) = deadline.wait(scope.pop())? {
            return Err(format!("wgpu error scope: {error}"));
        }
    }
    outcome?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_future_uses_finite_deadline_without_gpu() {
        assert!(
            Deadline(Instant::now() + Duration::from_millis(2))
                .wait(std::future::pending::<()>())
                .is_err()
        );
    }
}
