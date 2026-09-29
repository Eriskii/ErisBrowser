#![forbid(unsafe_code)]

use eris_vulkan_raster_prototype::{
    DrawKind, MAX_GPU_BUFFER_BYTES, PARAM_STRIDE, Plan, Result, alpha_fixtures, fixtures,
    image_fixtures,
};
use std::{
    future::Future,
    pin::pin,
    process::ExitCode,
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
struct Deadline(Instant);
impl Deadline {
    fn remaining(&self) -> Result<Duration> {
        self.0
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .map(|d| d.min(API_TIMEOUT))
            .ok_or_else(|| "whole application deadline".into())
    }
    fn wait<F: Future>(&self, future: F) -> Result<F::Output> {
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

struct Kernels {
    rectangle: wgpu::ComputePipeline,
    rectangle_layout: wgpu::BindGroupLayout,
    image: wgpu::ComputePipeline,
    image_layout: wgpu::BindGroupLayout,
}

fn raster(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    kernels: &Kernels,
    p: &Plan,
    expected: &[u32],
    deadline: &Deadline,
) -> Result<()> {
    deadline.remaining()?;
    let frame = p.frame();
    let pixel_bytes = u64::from(frame.width) * u64::from(frame.height) * 4;
    let metadata = p.parameters();
    let input = p.input_bytes();
    let actual_buffer_bytes = pixel_bytes * 2 + metadata.len() as u64 + input.len() as u64;
    if actual_buffer_bytes != p.gpu_buffer_bytes() || actual_buffer_bytes > MAX_GPU_BUFFER_BYTES {
        return Err("explicit GPU allocation budget mismatch".into());
    }
    if expected.len() as u64 * 4 != pixel_bytes {
        return Err("fixture length mismatch".into());
    }
    let alignment = device.limits().min_uniform_buffer_offset_alignment;
    if alignment == 0 || !(PARAM_STRIDE as u32).is_multiple_of(alignment) {
        return Err("unexpected uniform alignment".into());
    }
    let output = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("GPU-written packed RGB only"),
        size: pixel_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("bounded readback"),
        size: pixel_bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let parameters = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("immutable ordered draw metadata"),
        size: metadata.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("rectangle output and uniform window"),
        layout: &kernels.rectangle_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: output.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &parameters,
                    offset: 0,
                    size: wgpu::BufferSize::new(32),
                }),
            },
        ],
    });
    // Rectangle-only plans allocate no input buffer or dummy binding. An image
    // plan uploads original source pixels and separable source-index tables,
    // never a CPU-painted target. Output deliberately has no COPY_DST usage.
    let image_input = if p.has_images() {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bounded immutable source pixels and index tables"),
            size: input.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("image output, uniform window and input arena"),
            layout: &kernels.image_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &parameters,
                        offset: 0,
                        size: wgpu::BufferSize::new(64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });
        queue.write_buffer(&buffer, 0, input);
        Some((buffer, group))
    } else {
        None
    };
    queue.write_buffer(&parameters, 0, metadata);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ordered custom compute rasterization"),
    });
    for (index, draw) in p.draws().iter().enumerate() {
        deadline.remaining()?;
        // Separate pass boundary for every draw; storage read/write transitions are
        // tracked/barriered by wgpu-core. Never dispatch overlapping writers in
        // one dispatch or rely on a workgroup barrier for cross-dispatch order.
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("one ordered source-over draw"),
            timestamp_writes: None,
        });
        let (pipeline, draw_group) = match draw.kind() {
            DrawKind::Rectangle => (&kernels.rectangle, &group),
            DrawKind::Image => (
                &kernels.image,
                &image_input.as_ref().ok_or("missing image arena")?.1,
            ),
        };
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, draw_group, &[(index * PARAM_STRIDE) as u32]);
        let (x, y) = draw.groups();
        pass.dispatch_workgroups(x, y, 1);
    }
    encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, pixel_bytes);
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
    output.destroy();
    readback.destroy();
    parameters.destroy();
    if let Some((buffer, _)) = image_input {
        buffer.destroy();
    }
    comparison
}
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).take(4).collect();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => {
            Some(index.parse::<usize>().map_err(|_| "adapter index")?)
        }
        _ => return Err("usage: eris-vulkan-raster-prototype --list | --adapter INDEX".into()),
    };
    // Reject the entire CPU-planned input set before any Vulkan operation.
    let fixtures = fixtures::fixtures();
    let mut images = image_fixtures::fixtures();
    images.extend(alpha_fixtures::fixtures());
    let plans = fixtures
        .iter()
        .map(|f| Ok((f.name, f.expected.as_slice(), f.plan()?)))
        .chain(
            images
                .iter()
                .map(|f| Ok((f.name, f.expected.as_slice(), f.plan()?))),
        )
        .collect::<Result<Vec<_>>>()?;
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
    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let oom = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bounded rectangle bindings"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(4),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(32),
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("explicit compute layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("custom WGSL rectangle rasterizer"),
        source: wgpu::ShaderSource::Wgsl(include_str!("rect.wgsl").into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("integer coverage and source-over blending"),
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("rectangle"),
        compilation_options: Default::default(),
        cache: None,
    });
    let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bounded image bindings"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(4),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(4),
                },
                count: None,
            },
        ],
    });
    let image_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("explicit image compute layout"),
        bind_group_layouts: &[Some(&image_layout)],
        immediate_size: 0,
    });
    let image_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("custom WGSL source-over image rasterizer"),
        source: wgpu::ShaderSource::Wgsl(include_str!("image.wgsl").into()),
    });
    let image_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("integer nearest-neighbor gathering and blending"),
        layout: Some(&image_pipeline_layout),
        module: &image_module,
        entry_point: Some("image"),
        compilation_options: Default::default(),
        cache: None,
    });
    let kernels = Kernels {
        rectangle: pipeline,
        rectangle_layout: layout,
        image: image_pipeline,
        image_layout,
    };
    let outcome: Result<()> = (|| {
        for (name, expected, plan) in &plans {
            raster(&device, &queue, &kernels, plan, expected, &deadline)
                .map_err(|e| format!("{name}: {e}"))?;
            let frame = plan.frame();
            println!(
                "PASS {} {}x{} draws={} invocations={} gpu_buffers={} compared_bytes={} exact=true",
                name,
                frame.width,
                frame.height,
                plan.draws().len(),
                plan.invocations(),
                plan.gpu_buffer_bytes(),
                expected.len() * 4
            );
        }
        Ok(())
    })();
    for scope in [internal, oom, validation] {
        if let Some(error) = deadline.wait(scope.pop())? {
            return Err(format!("wgpu error scope: {error}"));
        }
    }
    outcome?;
    println!(
        "COMPLETE adapter={index} fixtures={} exact=true custom_wgsl=true",
        plans.len()
    );
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("FAIL {e}");
            ExitCode::FAILURE
        }
    }
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
