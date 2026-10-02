//! Integer packed-RGB conversion for a caller-owned native presentation path.
//!
//! This module creates no surface, submission, readback, or expected pixels. The
//! caller owns the device/queue/encoder, error scopes, cancellation and completion
//! proof. Use the same device and queue as the source [`EncodedFrame`]. Preflight
//! with [`SurfaceLayout::for_plan`] before encoding that source. Only Native plans
//! reserve this conversion's padded storage and work; Probe plans are refused.
//!
//! A [`ConvertedFrame`] owns its source frame and all conversion dependencies.
//! Retain it until every submitted use completes. An encoding error requires
//! discarding the encoder and retiring any already queued source metadata writes;
//! ordinary Rust drop never explicitly destroys buffers still used by wgpu.
use crate::{MAX_INVOCATIONS, Plan, Profile, Result, gpu::EncodedFrame};
use std::sync::Arc;

mod reuse;
pub use reuse::{NativeBufferLease, NativeEncoder, NativeRequirements};

const UNIFORM_BYTES: u64 = 16;

/// Checked shape and the exact storage/work already reserved by a Native plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceLayout {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    padded_bytes_per_row: u32,
    storage_bytes: u64,
    owned_buffer_bytes: u64,
    planned_buffer_bytes: u64,
    conversion_invocations: u64,
    invocations: u64,
    red_shift: u32,
}

/// A private copy of admitted metadata makes preflight arithmetic testable
/// without creating a device or manufacturing a public, unchecked Plan.
#[derive(Clone, Copy)]
pub(super) struct Admission {
    pub profile: Profile,
    pub width: u32,
    pub height: u32,
    pub output_bytes: u64,
    pub core_owned_bytes: u64,
    pub conversion_buffer_bytes: u64,
    pub planned_buffer_bytes: u64,
    pub conversion_invocations: u64,
    pub invocations: u64,
}

impl SurfaceLayout {
    pub fn for_plan(plan: &Plan, format: wgpu::TextureFormat) -> Result<Self> {
        let frame = plan.frame();
        let output_bytes = u64::from(frame.width)
            .checked_mul(u64::from(frame.height))
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or("surface input byte overflow")?;
        let core_owned_bytes = output_bytes
            .checked_add(plan.parameters().len() as u64)
            .and_then(|bytes| bytes.checked_add(plan.input_bytes().len() as u64))
            .ok_or("surface plan storage mismatch")?;
        Self::checked(
            Admission {
                profile: plan.profile(),
                width: frame.width,
                height: frame.height,
                output_bytes,
                core_owned_bytes,
                conversion_buffer_bytes: plan.conversion_buffer_bytes(),
                planned_buffer_bytes: plan.gpu_buffer_bytes(),
                conversion_invocations: plan.conversion_invocations(),
                invocations: plan.invocations(),
            },
            format,
        )
    }

    pub fn for_frame(frame: &EncodedFrame, format: wgpu::TextureFormat) -> Result<Self> {
        let size = frame.frame();
        let layout = Self::checked(
            Admission {
                profile: frame.profile(),
                width: size.width,
                height: size.height,
                output_bytes: frame.output_bytes(),
                core_owned_bytes: frame.owned_buffer_bytes(),
                conversion_buffer_bytes: frame.conversion_buffer_bytes(),
                planned_buffer_bytes: frame.planned_buffer_bytes(),
                conversion_invocations: frame.conversion_invocations(),
                invocations: frame.invocations(),
            },
            format,
        )?;
        if frame.output_buffer().size() != frame.output_bytes()
            || !frame
                .output_buffer()
                .usage()
                .contains(wgpu::BufferUsages::STORAGE)
        {
            return Err("surface source buffer mismatch".into());
        }
        Ok(layout)
    }

    pub(super) fn checked(admission: Admission, format: wgpu::TextureFormat) -> Result<Self> {
        if admission.profile != Profile::Native {
            return Err("surface conversion requires a Native plan".into());
        }
        // These are unorm textures with an opaque byte alpha, not sRGB texture
        // formats. The native surface's color-space contract is caller-owned.
        let red_shift = match format {
            wgpu::TextureFormat::Bgra8Unorm => 16,
            wgpu::TextureFormat::Rgba8Unorm => 0,
            _ => return Err("unsupported surface conversion format".into()),
        };
        let conversion_invocations = admission
            .profile
            .conversion_invocations(admission.width, admission.height)?;
        if conversion_invocations == 0
            || admission.conversion_invocations != conversion_invocations
            || admission.invocations < conversion_invocations
            || admission.invocations > MAX_INVOCATIONS
        {
            return Err("surface conversion work reservation mismatch".into());
        }
        let packed_row = admission
            .width
            .checked_mul(4)
            .ok_or("surface row byte overflow")?;
        let padded_bytes_per_row = packed_row
            .checked_add(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1)
            .map(|bytes| bytes / wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            .and_then(|rows| rows.checked_mul(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT))
            .ok_or("surface row alignment overflow")?;
        let packed_bytes = u64::from(packed_row)
            .checked_mul(u64::from(admission.height))
            .ok_or("surface input byte overflow")?;
        let storage_bytes = u64::from(padded_bytes_per_row)
            .checked_mul(u64::from(admission.height))
            .ok_or("surface output byte overflow")?;
        let owned_buffer_bytes = storage_bytes
            .checked_add(UNIFORM_BYTES)
            .ok_or("surface conversion storage overflow")?;
        if admission.output_bytes != packed_bytes
            || admission.core_owned_bytes < packed_bytes
            || admission.conversion_buffer_bytes != owned_buffer_bytes
            || admission.core_owned_bytes.checked_add(owned_buffer_bytes)
                != Some(admission.planned_buffer_bytes)
            || admission.planned_buffer_bytes > admission.profile.max_gpu_buffer_bytes()
        {
            return Err("surface conversion storage reservation mismatch".into());
        }
        Ok(Self {
            width: admission.width,
            height: admission.height,
            format,
            padded_bytes_per_row,
            storage_bytes,
            owned_buffer_bytes,
            planned_buffer_bytes: admission.planned_buffer_bytes,
            conversion_invocations,
            invocations: admission.invocations,
            red_shift,
        })
    }

    pub fn width(self) -> u32 {
        self.width
    }
    pub fn height(self) -> u32 {
        self.height
    }
    pub fn format(self) -> wgpu::TextureFormat {
        self.format
    }
    pub fn padded_bytes_per_row(self) -> u32 {
        self.padded_bytes_per_row
    }
    /// Padded output buffer only. The conversion uniform is reported separately
    /// by `owned_buffer_bytes`; optional caller readback is in neither figure.
    pub fn storage_bytes(self) -> u64 {
        self.storage_bytes
    }
    /// Conversion output plus its exactly 16-byte uniform.
    pub fn owned_buffer_bytes(self) -> u64 {
        self.owned_buffer_bytes
    }
    /// All core and conversion buffers; excludes caller readback/driver storage.
    pub fn planned_buffer_bytes(self) -> u64 {
        self.planned_buffer_bytes
    }
    pub fn conversion_invocations(self) -> u64 {
        self.conversion_invocations
    }
    /// Entire admitted frame work, including this conversion exactly once.
    pub fn invocations(self) -> u64 {
        self.invocations
    }
    pub(super) fn groups(self) -> (u32, u32) {
        (self.width.div_ceil(8), self.height.div_ceil(8))
    }
    pub(super) fn uniform(self) -> [u8; UNIFORM_BYTES as usize] {
        let words = [
            self.width,
            self.height,
            self.padded_bytes_per_row / 4,
            self.red_shift,
        ];
        let mut bytes = [0; UNIFORM_BYTES as usize];
        for (index, word) in words.into_iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }
    pub(super) fn check_device(self, limits: &wgpu::Limits) -> Result<()> {
        let (x, y) = self.groups();
        // Source storage is no larger than the padded output storage.
        if self.storage_bytes > limits.max_buffer_size
            || self.storage_bytes > limits.max_storage_buffer_binding_size
            || UNIFORM_BYTES > limits.max_uniform_buffer_binding_size
            || x > limits.max_compute_workgroups_per_dimension
            || y > limits.max_compute_workgroups_per_dimension
            || limits.max_compute_workgroup_size_x < 8
            || limits.max_compute_workgroup_size_y < 8
            || limits.max_compute_invocations_per_workgroup < 64
        {
            return Err("surface conversion exceeds device limits".into());
        }
        Ok(())
    }
}

struct Kernel {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

/// Owns the integer conversion pipeline. Establish caller error scopes and
/// cancellation policy before construction; synchronous driver work cannot be
/// interrupted here.
pub struct SurfaceConverter {
    kernel: Arc<Kernel>,
}
impl SurfaceConverter {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("packed RGB surface conversion bindings"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(UNIFORM_BYTES),
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("integer surface conversion pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("opaque integer RGB conversion"),
            source: wgpu::ShaderSource::Wgsl(include_str!("surface.wgsl").into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("packed RGB to padded native surface bytes"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("surface_convert"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            kernel: Arc::new(Kernel { layout, pipeline }),
        }
    }

    /// Append conversion after the source's ordered draws in the SAME encoder.
    ///
    /// Checks run before allocation and before upload/dispatch. On error discard
    /// that encoder and retire queued writes from its earlier source encoding.
    /// The source is consumed even on error, but ordinary drop does not destroy
    /// buffers; recorded wgpu commands retain their references. This method never
    /// submits, polls, pops scopes or reads pixels.
    pub fn encode(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source: EncodedFrame,
        format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        mut before_encode: impl FnMut() -> Result<()>,
    ) -> Result<ConvertedFrame> {
        before_encode()?;
        let layout = SurfaceLayout::for_frame(&source, format)?;
        layout.check_device(&device.limits())?;
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("opaque padded native surface pixels"),
            size: layout.storage_bytes(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let parameters = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("immutable surface conversion metadata"),
            size: UNIFORM_BYTES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("native surface conversion input/output"),
            layout: &self.kernel.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: source.output_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: parameters.as_entire_binding(),
                },
            ],
        });
        before_encode()?;
        queue.write_buffer(&parameters, 0, &layout.uniform());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("opaque native surface conversion"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.kernel.pipeline);
            pass.set_bind_group(0, &group, &[]);
            let (x, y) = layout.groups();
            pass.dispatch_workgroups(x, y, 1);
        }
        Ok(ConvertedFrame {
            source,
            output,
            parameters,
            layout,
            _group: group,
            _kernel: Arc::clone(&self.kernel),
        })
    }
}

/// Owns the source frame and the padded conversion buffers through completion.
/// No readback buffer or native surface is owned by this lease.
pub struct ConvertedFrame {
    source: EncodedFrame,
    output: wgpu::Buffer,
    parameters: wgpu::Buffer,
    layout: SurfaceLayout,
    _group: wgpu::BindGroup,
    _kernel: Arc<Kernel>,
}
impl ConvertedFrame {
    pub fn output_buffer(&self) -> &wgpu::Buffer {
        &self.output
    }
    pub fn layout(&self) -> SurfaceLayout {
        self.layout
    }
    pub fn source_frame(&self) -> &EncodedFrame {
        &self.source
    }
    /// Explicit destruction requires caller-proved completion of ALL recorded
    /// and submitted uses of both source and conversion. This is not a wait or
    /// completion check. Ordinary drop delegates resource lifetime to wgpu.
    pub fn destroy_after_completion(self) {
        self.output.destroy();
        self.parameters.destroy();
        self.source.destroy_after_completion();
    }
}
