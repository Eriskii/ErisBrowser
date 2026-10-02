//! Bounded ordered compute drawing on caller-owned wgpu devices.
//!
//! This module creates no instance, adapter, surface, device, submission, or
//! readback. The caller owns error scopes, cancellation policy, submission and
//! completion. Push the required validation/OOM/internal scopes before creating
//! a [`Rasterizer`], and collect them after the caller's work completes.
//!
//! Use the same device and queue for pipeline creation, encoding and submission.
//! Retain each [`EncodedFrame`] through submission completion. Its output is
//! exposed by a shared buffer reference for subsequent GPU reads or copying;
//! callers remain responsible for their own commands and shared wgpu handles.
//! Probe admission retains its original second packed target. Native admission
//! reserves padded conversion storage and its uniform instead; any native
//! readback requires a separate caller ledger. Driver/allocator overhead is excluded.
use crate::{DrawKind, Frame, PARAM_STRIDE, Plan, Profile, Result};
use std::sync::Arc;

struct Kernels {
    rectangle: wgpu::ComputePipeline,
    rectangle_layout: wgpu::BindGroupLayout,
    image: wgpu::ComputePipeline,
    image_layout: wgpu::BindGroupLayout,
    glyph: Option<wgpu::ComputePipeline>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BufferAccounting {
    output_bytes: u64,
    owned_buffer_bytes: u64,
    planned_buffer_bytes: u64,
}
impl BufferAccounting {
    fn checked(
        profile: Profile,
        output: u64,
        parameters: u64,
        input: u64,
        reserved: u64,
        planned: u64,
    ) -> Result<Self> {
        let owned = output
            .checked_add(parameters)
            .and_then(|bytes| bytes.checked_add(input));
        let total = owned.and_then(|bytes| bytes.checked_add(reserved));
        if total != Some(planned) || planned > profile.max_gpu_buffer_bytes() {
            return Err("explicit GPU allocation budget mismatch".into());
        }
        Ok(Self {
            output_bytes: output,
            owned_buffer_bytes: owned.ok_or("explicit GPU allocation budget mismatch")?,
            planned_buffer_bytes: planned,
        })
    }
    fn for_plan(plan: &Plan, alignment: u32, glyphs: bool) -> Result<Self> {
        // Refuse unsupported dispatches before allocating any frame buffers.
        if plan.has_glyphs() && !glyphs {
            return Err("missing glyph pipeline".into());
        }
        let frame = plan.frame();
        let output_bytes = u64::from(frame.width) * u64::from(frame.height) * 4;
        let reserved = match plan.profile() {
            Profile::Probe => output_bytes,
            Profile::Native => plan.conversion_buffer_bytes(),
        };
        let accounting = Self::checked(
            plan.profile(),
            output_bytes,
            plan.parameters().len() as u64,
            plan.input_bytes().len() as u64,
            reserved,
            plan.gpu_buffer_bytes(),
        )?;
        if alignment == 0 || !(PARAM_STRIDE as u32).is_multiple_of(alignment) {
            return Err("unexpected uniform alignment".into());
        }
        Ok(accounting)
    }
}

/// Pipelines for the existing packed-RGB plan format.
///
/// Rectangle and image pipelines are always present; glyph creation is optional.
/// Each encoded frame retains these pipelines independently of this handle.
/// Pipeline creation itself is synchronous driver work; caller deadlines cannot
/// interrupt it. Establish error scopes and check cancellation before calling.
pub struct Rasterizer {
    kernels: Arc<Kernels>,
}
impl Rasterizer {
    pub fn new(device: &wgpu::Device, glyphs: bool) -> Self {
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
        let image_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
        // Existing rectangle/image-only runs do not create an extra shader or
        // pipeline. Glyphs reuse the exact input/output/uniform binding layout.
        let glyph_pipeline = if glyphs {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("custom integer glyph coverage"),
                source: wgpu::ShaderSource::Wgsl(include_str!("glyph.wgsl").into()),
            });
            Some(
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("ordered glyph coverage and source-over"),
                    layout: Some(&image_pipeline_layout),
                    module: &module,
                    entry_point: Some("glyph"),
                    compilation_options: Default::default(),
                    cache: None,
                }),
            )
        } else {
            None
        };
        let kernels = Kernels {
            rectangle: pipeline,
            rectangle_layout: layout,
            image: image_pipeline,
            image_layout,
            glyph: glyph_pipeline,
        };
        Self {
            kernels: Arc::new(kernels),
        }
    }

    /// Upload immutable plan metadata and encode one ordered pass per draw.
    ///
    /// `before_draw` runs before preparation and again before every draw, so the
    /// caller can enforce its deadline or cancellation policy. Any error requires
    /// discarding the encoder: it may contain an ordered prefix of this frame.
    /// Queued metadata writes may already exist. This method does not submit,
    /// poll, pop error scopes, or wait, and synchronous driver calls cannot be
    /// interrupted by the callback.
    pub fn encode(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        p: &Plan,
        encoder: &mut wgpu::CommandEncoder,
        mut before_draw: impl FnMut() -> Result<()>,
    ) -> Result<EncodedFrame> {
        before_draw()?;
        let accounting = BufferAccounting::for_plan(
            p,
            device.limits().min_uniform_buffer_offset_alignment,
            self.kernels.glyph.is_some(),
        )?;
        let pixel_bytes = accounting.output_bytes;
        let metadata = p.parameters();
        let input = p.input_bytes();
        let kernels = &self.kernels;
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU-written packed RGB only"),
            size: pixel_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
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
        // Rectangle-only plans allocate no input buffer or dummy binding. Inputs
        // hold original source colors, coverage and index/row tables, never a
        // CPU-painted target. Output deliberately has no COPY_DST usage.
        let input_arena = if p.has_input() {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("bounded immutable colors, coverage and coordinate tables"),
                size: input.len() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("input-backed output, uniform window and input arena"),
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
        for (index, draw) in p.draws().iter().enumerate() {
            before_draw()?;
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
                    &input_arena.as_ref().ok_or("missing image arena")?.1,
                ),
                DrawKind::Glyph => (
                    kernels.glyph.as_ref().ok_or("missing glyph pipeline")?,
                    &input_arena.as_ref().ok_or("missing glyph arena")?.1,
                ),
            };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, draw_group, &[(index * PARAM_STRIDE) as u32]);
            let (x, y) = draw.groups();
            pass.dispatch_workgroups(x, y, 1);
        }
        Ok(EncodedFrame {
            frame: p.frame(),
            profile: p.profile(),
            conversion_invocations: p.conversion_invocations(),
            conversion_buffer_bytes: p.conversion_buffer_bytes(),
            invocations: p.invocations(),
            accounting,
            output,
            parameters,
            _rectangle_group: group,
            input_arena,
            _kernels: Arc::clone(&self.kernels),
        })
    }
}

/// Frame buffers and their binding/pipeline dependencies, retained for submission.
///
/// No CPU target pixels are uploaded. The output usage is STORAGE | COPY_SRC;
/// there is no COPY_DST usage or readback allocation in the core.
pub struct EncodedFrame {
    frame: Frame,
    profile: Profile,
    conversion_invocations: u64,
    conversion_buffer_bytes: u64,
    invocations: u64,
    accounting: BufferAccounting,
    output: wgpu::Buffer,
    parameters: wgpu::Buffer,
    _rectangle_group: wgpu::BindGroup,
    input_arena: Option<(wgpu::Buffer, wgpu::BindGroup)>,
    _kernels: Arc<Kernels>,
}
impl EncodedFrame {
    pub fn profile(&self) -> Profile {
        self.profile
    }
    pub fn conversion_invocations(&self) -> u64 {
        self.conversion_invocations
    }
    pub fn conversion_buffer_bytes(&self) -> u64 {
        self.conversion_buffer_bytes
    }
    pub fn invocations(&self) -> u64 {
        self.invocations
    }
    pub fn output_buffer(&self) -> &wgpu::Buffer {
        &self.output
    }
    pub fn frame(&self) -> Frame {
        self.frame
    }
    pub fn output_bytes(&self) -> u64 {
        self.accounting.output_bytes
    }
    /// Output + parameters + optional immutable inputs; excludes readback.
    pub fn owned_buffer_bytes(&self) -> u64 {
        self.accounting.owned_buffer_bytes
    }
    /// Full admitted charge, including Probe readback or Native conversion.
    pub fn planned_buffer_bytes(&self) -> u64 {
        self.accounting.planned_buffer_bytes
    }
    /// Explicitly destroy buffers AFTER the caller has confirmed all uses ended.
    ///
    /// This method does not submit, poll, wait for, or prove completion. Calling
    /// it too early can invalidate pending uses. Ordinary drop does not call
    /// Buffer::destroy; wgpu retains references required by recorded commands.
    pub fn destroy_after_completion(self) {
        self.output.destroy();
        self.parameters.destroy();
        if let Some((buffer, _)) = self.input_arena {
            buffer.destroy();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Command, MAX_GPU_BUFFER_BYTES, SourceMask, plan, plan_with_masks,
        plan_with_masks_for_profile,
    };

    #[test]
    fn two_target_admission_accepts_exact_cap_and_rejects_overflow_or_mismatch() {
        let input = MAX_GPU_BUFFER_BYTES - 8 - 256;
        let exact =
            BufferAccounting::checked(Profile::Probe, 4, 256, input, 4, MAX_GPU_BUFFER_BYTES)
                .unwrap();
        assert_eq!(exact.owned_buffer_bytes, MAX_GPU_BUFFER_BYTES - 4);
        assert_eq!(exact.output_bytes, 4);
        assert!(
            BufferAccounting::checked(
                Profile::Probe,
                4,
                256,
                input + 1,
                4,
                MAX_GPU_BUFFER_BYTES + 1
            )
            .is_err()
        );
        assert!(
            BufferAccounting::checked(Profile::Probe, 4, 256, input, 4, MAX_GPU_BUFFER_BYTES - 1)
                .is_err()
        );
        assert!(BufferAccounting::checked(Profile::Probe, u64::MAX, 256, 0, 0, 255).is_err());
        assert!(
            BufferAccounting::checked(Profile::Probe, u64::MAX / 2 + 1, 0, 0, u64::MAX / 2 + 1, 0)
                .is_err()
        );
    }

    #[test]
    fn clear_plan_keeps_second_target_charge_and_uniform_alignment_refusal() {
        let plan = plan(Frame::new(4, 2, 0), &[]).unwrap();
        let bytes = BufferAccounting::for_plan(&plan, 256, false).unwrap();
        assert_eq!(
            bytes,
            BufferAccounting {
                output_bytes: 32,
                owned_buffer_bytes: 288,
                planned_buffer_bytes: 320,
            }
        );
        assert_eq!(BufferAccounting::for_plan(&plan, 16, false).unwrap(), bytes);
        for alignment in [0, 3, 512] {
            assert_eq!(
                BufferAccounting::for_plan(&plan, alignment, false).unwrap_err(),
                "unexpected uniform alignment"
            );
        }
        assert!(!plan.has_input());
    }

    #[test]
    fn glyph_plan_requires_pipeline_before_frame_preparation() {
        let plan = plan_with_masks(
            Frame::new(1, 1, 0),
            &[Command::Glyph {
                source: 0,
                rows: 0,
                y: 0,
                rgba: [1, 2, 3, 255],
            }],
            &[],
            &[SourceMask {
                width: 1,
                height: 1,
                coverage: &[255],
            }],
            &[&[0]],
        )
        .unwrap();
        assert_eq!(
            BufferAccounting::for_plan(&plan, 256, false).unwrap_err(),
            "missing glyph pipeline"
        );
        let bytes = BufferAccounting::for_plan(&plan, 256, true).unwrap();
        assert_eq!(bytes.output_bytes, 4);
        assert_eq!(bytes.owned_buffer_bytes, 524);
        assert_eq!(bytes.planned_buffer_bytes, 528);
    }
    #[test]
    fn native_accounting_reserves_padded_conversion_and_uses_only_selected_cap() {
        let plan = plan_with_masks_for_profile(
            Profile::Native,
            Frame::new(1180, 880, 0),
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        let bytes = BufferAccounting::for_plan(&plan, 256, false).unwrap();
        assert_eq!(bytes.output_bytes, 4_153_600);
        assert_eq!(bytes.owned_buffer_bytes, 4_153_856);
        assert_eq!(plan.conversion_buffer_bytes(), 4_280_336);
        assert_eq!(bytes.planned_buffer_bytes, 8_434_192);
        assert!(
            BufferAccounting::checked(
                Profile::Probe,
                bytes.output_bytes,
                256,
                0,
                plan.conversion_buffer_bytes(),
                bytes.planned_buffer_bytes
            )
            .is_err()
        );
        // Synthetic accounting boundaries, not a claim that public source/work
        // limits can reach every byte of this absolute allocation ceiling.
        let cap = Profile::Native.max_gpu_buffer_bytes();
        assert!(
            BufferAccounting::checked(Profile::Native, 4, 256, cap - 4 - 256 - 272, 272, cap)
                .is_ok()
        );
        assert!(
            BufferAccounting::checked(
                Profile::Native,
                4,
                256,
                cap - 4 - 256 - 272 + 1,
                272,
                cap + 1
            )
            .is_err()
        );
    }
}
