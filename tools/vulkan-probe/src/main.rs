#![forbid(unsafe_code)]

use std::{
    future::Future,
    pin::pin,
    process::ExitCode,
    sync::{Arc, mpsc},
    task::{Context, Poll, Wake, Waker},
    thread,
    time::{Duration, Instant},
};

const MAX_WIDTH: u32 = 320;
const MAX_HEIGHT: u32 = 240;
const API_TIMEOUT: Duration = Duration::from_secs(5);
type Result<T> = std::result::Result<T, String>;

struct ThreadWake(thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

// This bounds Pending waits, not time inside a driver's future poll or Drop.
// The host runner therefore also imposes a separate process deadline.
fn wait_future<F: Future>(future: F, timeout: Duration) -> Result<F::Output> {
    let started = Instant::now();
    let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return Ok(value),
            Poll::Pending => {
                let remaining = timeout
                    .checked_sub(started.elapsed())
                    .ok_or("future exceeded application deadline")?;
                thread::park_timeout(remaining.min(Duration::from_millis(10)));
            }
        }
    }
}

fn row_layout(width: u32, height: u32) -> Result<(u32, u32, u64)> {
    if width == 0 || height == 0 || width > MAX_WIDTH || height > MAX_HEIGHT {
        return Err("dimensions outside the fixed 320x240 probe budget".into());
    }
    let packed = width.checked_mul(4).ok_or("row overflow")?;
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = packed.div_ceil(alignment) * alignment;
    let bytes = u64::from(padded) * u64::from(height);
    Ok((packed, padded, bytes))
}

fn pattern(width: u32, height: u32, frame: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            // Different channels/rows/frames detect swaps, flipped rows, stale
            // uploads and conversion. Include RGB at alpha zero and all alphas.
            pixels.extend_from_slice(&[
                ((x * 17 + y * 3 + frame * 43) & 255) as u8,
                ((x * 5 + y * 29 + frame * 67) & 255) as u8,
                ((x * 31 + y * 11 + frame * 97) & 255) as u8,
                ((x + y * 7 + frame * 19) & 255) as u8,
            ]);
        }
    }
    pixels
}

fn transfer_case(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) -> Result<()> {
    let (packed, padded, buffer_bytes) = row_layout(width, height)?;
    let extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("bounded RGBA8 transfer texture"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("bounded aligned readback"),
        size: buffer_bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    for frame in 0..3 {
        let expected = pattern(width, height, frame);
        // Queue uploads accept packed rows; encoder texture-to-buffer copies
        // require 256-byte row alignment. 319px forces actual padding.
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &expected,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(packed),
                rows_per_image: Some(height),
            },
            extent,
        );
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("upload/readback probe"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            extent,
        );
        let submission = queue.submit([encoder.finish()]);
        let (tx, rx) = mpsc::sync_channel(1);
        readback.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = tx.send(result);
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(API_TIMEOUT),
            })
            .map_err(|error| format!("bounded GPU poll: {error}"))?;
        rx.recv_timeout(API_TIMEOUT)
            .map_err(|error| format!("mapping callback deadline: {error}"))?
            .map_err(|error| format!("map failed: {error}"))?;
        let mapped = readback
            .get_mapped_range(..)
            .map_err(|error| format!("mapped view: {error}"))?;
        for y in 0..height as usize {
            let actual = &mapped[y * padded as usize..y * padded as usize + packed as usize];
            let expected_row = &expected[y * packed as usize..(y + 1) * packed as usize];
            if let Some(x) = actual.iter().zip(expected_row).position(|(a, b)| a != b) {
                return Err(format!(
                    "pixel mismatch: {width}x{height}, frame={frame}, row={y}, byte={x}, actual={}, expected={}",
                    actual[x], expected_row[x]
                ));
            }
        }
        drop(mapped);
        readback.unmap();
        println!(
            "PASS {width}x{height} frame={frame} compared_bytes={} packed_row={packed} readback_row={padded} readback_buffer={buffer_bytes}",
            expected.len()
        );
    }
    readback.destroy();
    texture.destroy();
    Ok(())
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => Some(
            index
                .parse::<usize>()
                .map_err(|_| "invalid adapter index")?,
        ),
        _ => return Err("usage: eris-vulkan-probe --list | --adapter INDEX".into()),
    };
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::VULKAN;
    // Do not consume WGPU_BACKEND or native surface/window configuration.
    let instance = wgpu::Instance::new(descriptor);
    assert_eq!(
        wgpu::Instance::enabled_backend_features(),
        wgpu::Backends::VULKAN
    );
    let adapters = wait_future(
        instance.enumerate_adapters(wgpu::Backends::VULKAN),
        API_TIMEOUT,
    )?;
    if adapters.is_empty() || adapters.len() > 16 {
        return Err(format!(
            "adapter count outside probe budget: {}",
            adapters.len()
        ));
    }
    for (index, adapter) in adapters.iter().enumerate() {
        let info = adapter.get_info();
        if info.backend != wgpu::Backend::Vulkan {
            return Err("unexpected non-Vulkan backend".into());
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
    let adapter = adapters
        .get(index)
        .ok_or("adapter index outside enumeration")?;
    println!("TEST adapter={index}");
    let (device, queue) = wait_future(
        adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Eris bounded Vulkan transfer probe"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }),
        API_TIMEOUT,
    )?
    .map_err(|error| format!("device request: {error}"))?;
    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let out_of_memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    transfer_case(&device, &queue, 320, 240)?;
    transfer_case(&device, &queue, 319, 239)?;
    for scope in [internal, out_of_memory, validation] {
        if let Some(error) = wait_future(scope.pop(), API_TIMEOUT)? {
            return Err(format!("wgpu error scope: {error}"));
        }
    }
    println!("COMPLETE adapter={index} comparisons=6 exact=true");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("FAIL {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn row_alignment_has_bounded_real_padding() {
        assert_eq!(row_layout(320, 240).unwrap(), (1280, 1280, 307200));
        assert_eq!(row_layout(319, 239).unwrap(), (1276, 1280, 305920));
        for dimensions in [
            (0, 240),
            (320, 0),
            (321, 240),
            (320, 241),
            (u32::MAX, u32::MAX),
        ] {
            assert!(row_layout(dimensions.0, dimensions.1).is_err());
        }
    }
    #[test]
    fn future_wait_has_a_pending_deadline() {
        assert!(wait_future(std::future::pending::<()>(), Duration::from_millis(2)).is_err());
    }
    #[test]
    fn pattern_covers_every_alpha_and_changes_between_frames() {
        let first = pattern(320, 240, 0);
        let mut alpha = [false; 256];
        let (pixels, remainder) = first.as_chunks::<4>();
        assert!(remainder.is_empty());
        for pixel in pixels {
            alpha[pixel[3] as usize] = true;
        }
        assert!(alpha.into_iter().all(|present| present));
        assert_ne!(first, pattern(320, 240, 1));
    }
}
