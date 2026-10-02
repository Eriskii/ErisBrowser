//! Offscreen multi-frame checks of complete native buffer leases. No surface,
//! reference upload, timing measurements or production cache counters are added.
use crate::{Result, gpu::Deadline, reuse_fixtures::Fixture, surface_gpu::compare};
use eris_raster_core::surface::{NativeBufferLease, NativeEncoder};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

#[derive(Default, Debug)]
pub struct Counts {
    pub frames: usize,
    pub allocations: usize,
    pub reuses: usize,
    pub evictions: usize,
    pub compared_bytes: u64,
}

// Collect all three scopes, preserving the first error even after a timeout.
fn scopes(
    device: &wgpu::Device,
    deadline: &Deadline,
    work: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let oom = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    let mut result = work();
    for scope in [internal, oom, validation] {
        let status = deadline.wait(scope.pop()).and_then(|error| match error {
            Some(error) => Err(format!("reuse checker scope: {error}")),
            None => Ok(()),
        });
        if result.is_ok() {
            result = status;
        }
    }
    result
}

fn cancellation_checks(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    native: &NativeEncoder,
    case: &Fixture,
    deadline: &Deadline,
) -> Result<()> {
    let format = wgpu::TextureFormat::Bgra8Unorm;
    let required = native.requirements(&case.plan, format)?;
    // Initial guard, each ordered draw guard, and the conversion guard.
    let checkpoints = case.plan.draws().len() + 2;
    for cut in 1..=checkpoints {
        let mut lease = None;
        let mut retired = false;
        let result = scopes(device, deadline, || {
            lease = Some(native.allocate(&required, || deadline.remaining().map(|_| ()))?);
            let lease = lease.as_mut().ok_or("cancellation lease")?;
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("discarded cancellation prefix"),
            });
            let mut calls = 0;
            let encoded = native.encode(lease, &case.plan, format, &mut encoder, || {
                calls += 1;
                if calls == cut {
                    Err("injected cancellation".into())
                } else {
                    deadline.remaining().map(|_| ())
                }
            });
            drop(encoder);
            // Conservative flush even for the first, pre-upload guard.
            let submission = queue.submit([]);
            device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(submission),
                    timeout: Some(deadline.remaining()?),
                })
                .map_err(|e| format!("cancellation retirement: {e}"))?;
            retired = true;
            if encoded.as_ref().err().map(String::as_str) != Some("injected cancellation")
                || calls != cut
                || lease.is_compatible(&required)
                || lease.output_buffer().is_ok()
                || lease.mark_reusable_after_completion().is_ok()
            {
                return Err("cancelled lease became reusable or lost its error".into());
            }
            Ok(())
        });
        if retired && let Some(lease) = lease.take() {
            lease.destroy_after_completion();
        }
        result?;
        println!(
            "PASS cancellation checkpoint={cut} flush_submissions=1 retired=true reusable=false scopes=3"
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    native: &NativeEncoder,
    cached: &mut Option<NativeBufferLease>,
    case: &Fixture,
    format: wgpu::TextureFormat,
    deadline: &Deadline,
    counts: &mut Counts,
    expect_reuse: bool,
) -> Result<()> {
    let required = native.requirements(&case.plan, format)?;
    let layout = required.layout();
    if required.buffer_bytes() != case.plan.gpu_buffer_bytes()
        || required
            .buffer_bytes()
            .checked_add(layout.storage_bytes())
            .is_none_or(|n| n > 24 * 1024 * 1024)
    {
        return Err("reuse checker explicit allocation budget".into());
    }
    let reused = cached
        .as_ref()
        .is_some_and(|lease| lease.is_compatible(&required));
    if reused != expect_reuse {
        return Err(format!(
            "{}: expected reuse={expect_reuse}, actual={reused}",
            case.name
        ));
    }
    if !reused && let Some(old) = cached.take() {
        old.destroy_after_completion();
        counts.evictions += 1;
    }
    // Keep the lease outside the error scopes/encoding closure, including Err.
    let mut owned = cached.take();
    let mut readback = None;
    let mut retired = false;
    let result = scopes(device, deadline, || {
        if owned.is_none() {
            owned = Some(native.allocate(&required, || deadline.remaining().map(|_| ()))?);
            counts.allocations += 1;
        } else {
            counts.reuses += 1;
        }
        let lease = owned.as_mut().ok_or("reuse checker missing lease")?;
        if lease.buffer_bytes() != required.buffer_bytes() {
            return Err("reuse checker lease charge mismatch".into());
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("changed native reuse frame"),
        });
        let encoded = native.encode(lease, &case.plan, format, &mut encoder, || {
            deadline.remaining().map(|_| ())
        });
        if let Err(error) = encoded {
            // No incomplete draw prefix is submitted. A queued-write prefix
            // must still have one tracked empty submission before safe drop.
            drop(encoder);
            let submission = queue.submit([]);
            retired = device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(submission),
                    timeout: Some(deadline.remaining()?),
                })
                .is_ok();
            return Err(error);
        }
        let target = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uncached separately charged reuse readback"),
            size: layout.storage_bytes(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(
            lease.output_buffer()?,
            0,
            &target,
            0,
            layout.storage_bytes(),
        );
        readback = Some(target);
        let target = readback.as_ref().ok_or("reuse checker readback")?;
        let submission = queue.submit([encoder.finish()]);
        let (tx, rx) = mpsc::sync_channel(1);
        target.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = tx.send(result);
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(deadline.remaining()?),
            })
            .map_err(|e| format!("reuse check poll: {e}"))?;
        retired = true;
        rx.recv_timeout(deadline.remaining()?)
            .map_err(|e| format!("reuse callback: {e}"))?
            .map_err(|e| format!("reuse mapping: {e}"))?;
        let comparison = {
            let mapped = target
                .get_mapped_range(..)
                .map_err(|e| format!("reuse map: {e}"))?;
            compare(
                &mapped,
                &case.expected,
                case.plan.frame().width,
                case.plan.frame().height,
                layout.padded_bytes_per_row(),
                format,
            )
        };
        target.unmap();
        comparison
    });
    if let Some(target) = readback.take() {
        if retired {
            target.destroy();
        }
        drop(target);
    }
    if let Err(error) = result {
        if retired && let Some(lease) = owned.take() {
            lease.destroy_after_completion();
        }
        return Err(error);
    }
    deadline.remaining()?;
    let mut lease = owned.take().ok_or("reuse checker completed lease")?;
    lease.mark_reusable_after_completion()?;
    if !lease.is_compatible(&required) || cached.is_some() {
        return Err("reuse checker completed compatibility".into());
    }
    *cached = Some(lease);
    counts.frames += 1;
    let compared = case.expected.len() as u64 * 4;
    counts.compared_bytes += compared;
    println!(
        "PASS reuse-{} format={format:?} width={} height={} reused={reused} planned_bytes={} compared_bytes={compared} reference=literal exact=true",
        case.name,
        case.plan.frame().width,
        case.plan.frame().height,
        required.buffer_bytes()
    );
    Ok(())
}

/// Checks both formats, format/size/input changes, repeated complete overwrites
/// and a separate context over the same Device/Queue. Counts are harness-local
/// observations of allocation/checkout/eviction operations, not driver metrics.
pub fn run(selected: Option<usize>, cases: &[Fixture]) -> Result<Counts> {
    if cases.len() != 9 || cases.iter().any(|case| case.expected.len() > 32) {
        return Err("reuse checker fixed fixture bounds".into());
    }
    let deadline = Deadline(Instant::now() + Duration::from_secs(20));
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
    for (index, adapter) in adapters.iter().enumerate() {
        let info = adapter.get_info();
        if info.backend != wgpu::Backend::Vulkan {
            return Err("non-Vulkan adapter".into());
        }
        println!(
            "ADAPTER {index} vendor={:#x} device={:#x} type={:?} backend={:?} name={:?} driver={:?} info={:?}",
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
        return Ok(Counts::default());
    };
    let adapter = adapters.get(index).ok_or("adapter outside inventory")?;
    let (device, queue) = deadline
        .wait(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("native buffer reuse validation"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }))?
        .map_err(|e| format!("device: {e}"))?;
    let mut native = None;
    scopes(&device, &deadline, || {
        native = Some(NativeEncoder::new(&device, &queue, || {
            deadline.remaining().map(|_| ())
        })?);
        Ok(())
    })?;
    let mut native = native.ok_or("reuse checker native context")?;
    cancellation_checks(&device, &queue, &native, &cases[0], &deadline)?;
    let mut cached = None;
    let mut counts = Counts::default();
    for format in [
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Rgba8Unorm,
    ] {
        // Equal-size input rewrites repeat A/B/C twice; no omitted warmup frames.
        for (case, reuse) in [
            (0, false),
            (1, true),
            (2, true),
            (0, true),
            (1, true),
            (2, true),
            (3, false),
            (4, true),
            (5, false),
            (6, true),
            (7, false),
            (8, false),
        ] {
            frame(
                &device,
                &queue,
                &native,
                &mut cached,
                &cases[case],
                format,
                &deadline,
                &mut counts,
                reuse,
            )?;
        }
        // Keep target/parameter sizes identical while changing only format.
        let other = match format {
            wgpu::TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            _ => wgpu::TextureFormat::Bgra8Unorm,
        };
        frame(
            &device,
            &queue,
            &native,
            &mut cached,
            &cases[8],
            other,
            &deadline,
            &mut counts,
            false,
        )?;
    }
    let mut replacement = None;
    scopes(&device, &deadline, || {
        replacement = Some(NativeEncoder::new(&device, &queue, || {
            deadline.remaining().map(|_| ())
        })?);
        Ok(())
    })?;
    native = replacement.ok_or("reuse checker replacement context")?;
    frame(
        &device,
        &queue,
        &native,
        &mut cached,
        &cases[8],
        wgpu::TextureFormat::Bgra8Unorm,
        &deadline,
        &mut counts,
        false,
    )?;
    frame(
        &device,
        &queue,
        &native,
        &mut cached,
        &cases[8],
        wgpu::TextureFormat::Bgra8Unorm,
        &deadline,
        &mut counts,
        true,
    )?;
    if let Some(lease) = cached.take() {
        lease.destroy_after_completion();
        counts.evictions += 1;
    }
    if counts.frames != 28
        || counts.allocations != 13
        || counts.reuses != 15
        || counts.evictions != counts.allocations
    {
        return Err(format!("reuse checker lifecycle totals: {counts:?}"));
    }
    Ok(counts)
}
