//! Offscreen validation of native-profile raster and packed surface conversion.
//! This harness creates no window or acquired surface. References never enter
//! either GPU encoder; only this caller performs readback and comparison.
use crate::{Plan, Result, gpu::Deadline};
use eris_raster_core::{
    gpu::Rasterizer,
    surface::{SurfaceConverter, SurfaceLayout},
};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

const MAX_CASES: usize = 29;
const ACTIVE_GPU_BYTES: u64 = 24 * 1024 * 1024;
const RETAINED_REFERENCE_BYTES: usize = 32 * 1024 * 1024;

#[allow(clippy::too_many_arguments)]
fn execute(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    rasterizer: &Rasterizer,
    converter: &SurfaceConverter,
    plan: &Plan,
    expected: &[u32],
    format: wgpu::TextureFormat,
    deadline: &Deadline,
) -> Result<()> {
    let layout = SurfaceLayout::for_plan(plan, format)?;
    if plan
        .gpu_buffer_bytes()
        .checked_add(layout.storage_bytes())
        .is_none_or(|bytes| bytes > ACTIVE_GPU_BYTES)
    {
        return Err("surface checker active GPU allocation budget".into());
    }
    deadline.remaining()?;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("native-profile raster and surface conversion check"),
    });
    let raster = rasterizer.encode(device, queue, plan, &mut encoder, || {
        deadline.remaining().map(|_| ())
    })?;
    let converted = converter.encode(device, queue, raster, format, &mut encoder, || {
        deadline.remaining().map(|_| ())
    })?;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("separately charged converted readback"),
        size: layout.storage_bytes(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(
        converted.output_buffer(),
        0,
        &readback,
        0,
        layout.storage_bytes(),
    );
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
        .map_err(|e| format!("surface check poll: {e}"))?;
    rx.recv_timeout(deadline.remaining()?)
        .map_err(|e| format!("surface check callback: {e}"))?
        .map_err(|e| format!("surface check mapping: {e}"))?;
    let comparison = {
        let mapped = readback
            .get_mapped_range(..)
            .map_err(|e| format!("surface check map: {e}"))?;
        compare(
            &mapped,
            expected,
            plan.frame().width,
            plan.frame().height,
            layout.padded_bytes_per_row(),
            format,
        )
    };
    readback.unmap();
    converted.destroy_after_completion();
    readback.destroy();
    comparison
}

pub(crate) fn compare(
    bytes: &[u8],
    expected: &[u32],
    width: u32,
    height: u32,
    stride: u32,
    format: wgpu::TextureFormat,
) -> Result<()> {
    if expected.len() as u64 != u64::from(width) * u64::from(height)
        || bytes.len() as u64 != u64::from(stride) * u64::from(height)
        || stride < width.checked_mul(4).ok_or("comparison row overflow")?
    {
        return Err("surface comparison lengths".into());
    }
    for y in 0..height as usize {
        for x in 0..width as usize {
            let rgb = expected[y * width as usize + x];
            let [b, g, r, _] = rgb.to_le_bytes();
            let reference = match format {
                wgpu::TextureFormat::Bgra8Unorm => [b, g, r, 255],
                wgpu::TextureFormat::Rgba8Unorm => [r, g, b, 255],
                _ => return Err("surface comparison format".into()),
            };
            let at = y * stride as usize + x * 4;
            if bytes[at..at + 4] != reference {
                return Err(format!(
                    "converted pixel ({x},{y}) {format:?}: actual={:?} expected={reference:?}",
                    &bytes[at..at + 4]
                ));
            }
        }
    }
    Ok(())
}

/// Every case runs in both byte formats. A successful comparison covers active
/// pixels including alpha; padded bytes are allocated but not pixel evidence.
pub fn run(
    selected: Option<usize>,
    plans: &[(&str, &[u32], &Plan)],
    mut passed: impl FnMut(&str, &Plan, wgpu::TextureFormat, u64) -> Result<()>,
) -> Result<()> {
    if plans.len() > MAX_CASES || (selected.is_some() && plans.is_empty()) {
        return Err("surface checker case count".into());
    }
    let mut retained = 0usize;
    for (_, expected, plan) in plans {
        let area = u64::from(plan.frame().width) * u64::from(plan.frame().height);
        if expected.len() as u64 != area {
            return Err("surface fixture pixel count".into());
        }
        retained = retained
            .checked_add(expected.len().checked_mul(4).ok_or("reference size")?)
            .filter(|bytes| *bytes <= RETAINED_REFERENCE_BYTES)
            .ok_or("surface reference budget")?;
        for format in [
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8Unorm,
        ] {
            let layout = SurfaceLayout::for_plan(plan, format)?;
            if plan
                .gpu_buffer_bytes()
                .checked_add(layout.storage_bytes())
                .is_none_or(|n| n > ACTIVE_GPU_BYTES)
            {
                return Err("surface checker active GPU allocation budget".into());
            }
        }
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
        return Ok(());
    };
    let adapter = adapters.get(index).ok_or("adapter outside inventory")?;
    let (device, queue) = deadline
        .wait(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("native-profile conversion validation"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }))?
        .map_err(|e| format!("device: {e}"))?;
    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let oom = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    let glyphs = plans.iter().any(|(_, _, p)| p.has_glyphs());
    let rasterizer = if plans.iter().any(|(_, _, p)| p.group_scratch_bytes() != 0) {
        Rasterizer::new_native(&device, glyphs)
    } else {
        Rasterizer::new(&device, glyphs)
    };
    let converter = SurfaceConverter::new(&device);
    let result = (|| {
        for (name, expected, plan) in plans {
            for format in [
                wgpu::TextureFormat::Bgra8Unorm,
                wgpu::TextureFormat::Rgba8Unorm,
            ] {
                execute(
                    &device,
                    &queue,
                    &rasterizer,
                    &converter,
                    plan,
                    expected,
                    format,
                    &deadline,
                )
                .map_err(|error| format!("{name}: {error}"))?;
                passed(name, plan, format, expected.len() as u64 * 4)?;
            }
        }
        Ok(())
    })();
    for scope in [internal, oom, validation] {
        if let Some(error) = deadline.wait(scope.pop())? {
            return Err(format!("surface checker scope: {error}"));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comparison_checks_both_byte_orders_and_alpha_with_padded_rows() {
        let mut bytes = [0u8; 512];
        for (format, first, second) in [
            (
                wgpu::TextureFormat::Bgra8Unorm,
                [0x56, 0x34, 0x12, 255],
                [0xef, 0xcd, 0xab, 255],
            ),
            (
                wgpu::TextureFormat::Rgba8Unorm,
                [0x12, 0x34, 0x56, 255],
                [0xab, 0xcd, 0xef, 255],
            ),
        ] {
            bytes[..4].copy_from_slice(&first);
            bytes[256..260].copy_from_slice(&second);
            compare(&bytes, &[0x123456, 0xabcdef], 1, 2, 256, format).unwrap();
            bytes[259] = 0;
            assert!(compare(&bytes, &[0x123456, 0xabcdef], 1, 2, 256, format).is_err());
        }
        assert!(compare(&[], &[], 1, 2, 256, wgpu::TextureFormat::Bgra8Unorm).is_err());
    }
}
