//! Additive Native-only RGBA16 pipelines and one optional scratch arena.
use super::*;

pub(super) const GROUP_UNIFORM_BYTES: u64 = 128;

pub(super) fn group_dispatch(draw: crate::Draw) -> bool {
    draw.target_is_group() || matches!(draw.kind(), DrawKind::GroupClear | DrawKind::GroupComposite)
}

pub(super) struct GroupKernels {
    plain_layout: wgpu::BindGroupLayout,
    input_layout: wgpu::BindGroupLayout,
    clear: wgpu::ComputePipeline,
    rectangle: wgpu::ComputePipeline,
    image: wgpu::ComputePipeline,
    glyph: Option<wgpu::ComputePipeline>,
    composite: wgpu::ComputePipeline,
}
impl GroupKernels {
    pub fn new(device: &wgpu::Device, glyphs: bool) -> Self {
        let entries = [
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
                    min_binding_size: wgpu::BufferSize::new(GROUP_UNIFORM_BYTES),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(8),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(4),
                },
                count: None,
            },
        ];
        let plain_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Native RGBA16 root, uniform and scratch bindings"),
            entries: &entries[..3],
        });
        let input_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Native RGBA16 bindings with immutable primitive input"),
            entries: &entries,
        });
        let plain_pipeline = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Native RGBA16 plain pipeline layout"),
            bind_group_layouts: &[Some(&plain_layout)],
            immediate_size: 0,
        });
        let input_pipeline = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Native RGBA16 input pipeline layout"),
            bind_group_layouts: &[Some(&input_layout)],
            immediate_size: 0,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Native bounded integer RGBA16 groups"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../group.wgsl").into()),
        });
        let pipeline = |entry, layout: &wgpu::PipelineLayout| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        Self {
            clear: pipeline("group_clear", &plain_pipeline),
            rectangle: pipeline("group_rectangle", &plain_pipeline),
            image: pipeline("group_image", &input_pipeline),
            glyph: glyphs.then(|| pipeline("group_glyph", &input_pipeline)),
            composite: pipeline("group_composite", &plain_pipeline),
            plain_layout,
            input_layout,
        }
    }
}

pub(super) struct GroupBuffers {
    scratch: wgpu::Buffer,
    plain: wgpu::BindGroup,
    input: Option<wgpu::BindGroup>,
}
impl GroupBuffers {
    /// Exact admission and callbacks precede all resource creation. Driver
    /// errors remain in caller scopes and all handles stay with frame/lease.
    pub fn allocate(
        device: &wgpu::Device,
        kernels: &GroupKernels,
        scratch_bytes: u64,
        output: &wgpu::Buffer,
        parameters: &wgpu::Buffer,
        input: Option<&wgpu::Buffer>,
    ) -> Self {
        let scratch = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Native cropped RGBA16 group scratch"),
            size: scratch_bytes,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let uniform = || {
            wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: parameters,
                offset: 0,
                size: wgpu::BufferSize::new(GROUP_UNIFORM_BYTES),
            })
        };
        let plain = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Native RGBA16 plain operations"),
            layout: &kernels.plain_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: scratch.as_entire_binding(),
                },
            ],
        });
        let input = input.map(|buffer| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Native RGBA16 primitive operations with inputs"),
                layout: &kernels.input_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: output.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: uniform(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: scratch.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: buffer.as_entire_binding(),
                    },
                ],
            })
        });
        Self {
            scratch,
            plain,
            input,
        }
    }

    pub fn matches(&self, bytes: u64, has_input: bool) -> bool {
        bytes != 0
            && buffer_matches(&self.scratch, bytes, wgpu::BufferUsages::STORAGE)
            && self.input.is_some() == has_input
    }

    pub fn bindings<'a>(
        &'a self,
        kernels: &'a GroupKernels,
        kind: DrawKind,
    ) -> Result<(&'a wgpu::ComputePipeline, &'a wgpu::BindGroup)> {
        Ok(match kind {
            DrawKind::Rectangle => (&kernels.rectangle, &self.plain),
            DrawKind::Image => (
                &kernels.image,
                self.input.as_ref().ok_or("missing group image arena")?,
            ),
            DrawKind::Glyph => (
                kernels
                    .glyph
                    .as_ref()
                    .ok_or("missing group glyph pipeline")?,
                self.input.as_ref().ok_or("missing group glyph arena")?,
            ),
            DrawKind::GroupClear => (&kernels.clear, &self.plain),
            DrawKind::GroupComposite => (&kernels.composite, &self.plain),
        })
    }

    pub fn destroy_after_completion(self) {
        self.scratch.destroy();
    }
}
