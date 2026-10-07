//! Private raster half of a complete reusable Native lease.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RasterRequirements {
    pub output: u64,
    pub parameters: u64,
    pub input: u64,
    pub scratch: u64,
    pub alignment: u32,
}
impl RasterRequirements {
    pub fn for_plan(
        plan: &Plan,
        limits: &wgpu::Limits,
        glyphs: bool,
        groups: bool,
    ) -> Result<Self> {
        let accounting = BufferAccounting::for_plan(
            plan,
            limits.min_uniform_buffer_offset_alignment,
            glyphs,
            groups,
        )?;
        let required = Self {
            output: accounting.output_bytes,
            parameters: plan.parameters().len() as u64,
            input: plan.input_bytes().len() as u64,
            scratch: plan.group_scratch_bytes(),
            alignment: limits.min_uniform_buffer_offset_alignment,
        };
        if plan.has_input() != (required.input != 0)
            || required.output > limits.max_buffer_size
            || required.output > limits.max_storage_buffer_binding_size
            || required.parameters > limits.max_buffer_size
            || required.input > limits.max_buffer_size
            || required.input > limits.max_storage_buffer_binding_size
            || required.scratch > limits.max_buffer_size
            || required.scratch > limits.max_storage_buffer_binding_size
            || limits.max_uniform_buffer_binding_size
                < if required.scratch != 0 {
                    groups::GROUP_UNIFORM_BYTES
                } else if plan.has_input() {
                    64
                } else {
                    32
                }
            || (required.scratch != 0
                && (limits.max_storage_buffers_per_shader_stage
                    < if plan.has_input() { 3 } else { 2 }
                    || limits.max_bindings_per_bind_group < if plan.has_input() { 4 } else { 3 }
                    || limits.max_bind_groups < 1
                    || limits.max_uniform_buffers_per_shader_stage < 1
                    || limits.max_dynamic_uniform_buffers_per_pipeline_layout < 1))
            || plan.draws().iter().any(|draw| {
                let (x, y) = draw.groups();
                x > limits.max_compute_workgroups_per_dimension
                    || y > limits.max_compute_workgroups_per_dimension
            })
        {
            return Err("native raster buffers exceed device limits".into());
        }
        Ok(required)
    }
}

pub(crate) struct RasterBuffers {
    output: wgpu::Buffer,
    parameters: wgpu::Buffer,
    rectangle_group: wgpu::BindGroup,
    input: Option<(wgpu::Buffer, wgpu::BindGroup)>,
    groups: Option<GroupBuffers>,
}
impl RasterBuffers {
    /// All fallible admission and callbacks precede this bounded construction.
    /// Driver failures remain in the caller's validation/OOM/internal scopes.
    pub fn allocate(
        device: &wgpu::Device,
        rasterizer: &Rasterizer,
        required: RasterRequirements,
    ) -> Self {
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reusable GPU-written packed RGB only"),
            size: required.output,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let parameters = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reusable ordered draw metadata"),
            size: required.parameters,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let rectangle_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("reusable rectangle output and uniform window"),
            layout: &rasterizer.kernels.rectangle_layout,
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
        let input = (required.input != 0).then(|| {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("reusable source colors, coverage and coordinates"),
                size: required.input,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("reusable input-backed draw bindings"),
                layout: &rasterizer.kernels.image_layout,
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
            (buffer, group)
        });
        let groups = (required.scratch != 0).then(|| {
            GroupBuffers::allocate(
                device,
                rasterizer
                    .kernels
                    .groups
                    .as_ref()
                    .expect("group pipelines admitted"),
                required.scratch,
                &output,
                &parameters,
                input.as_ref().map(|(buffer, _)| buffer),
            )
        });
        Self {
            output,
            parameters,
            rectangle_group,
            input,
            groups,
        }
    }

    pub fn matches(&self, required: RasterRequirements) -> bool {
        buffer_matches(
            &self.output,
            required.output,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        ) && buffer_matches(
            &self.parameters,
            required.parameters,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        ) && (match &self.input {
            Some((buffer, _)) => {
                required.input != 0
                    && buffer_matches(
                        buffer,
                        required.input,
                        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    )
            }
            None => required.input == 0,
        }) && match &self.groups {
            Some(groups) => groups.matches(required.scratch, required.input != 0),
            None => required.scratch == 0,
        }
    }

    pub fn output(&self) -> &wgpu::Buffer {
        &self.output
    }

    /// The caller retains this set even if a callback refuses a draw prefix.
    pub fn encode(
        &self,
        rasterizer: &Rasterizer,
        queue: &wgpu::Queue,
        plan: &Plan,
        encoder: &mut wgpu::CommandEncoder,
        check: &mut impl FnMut() -> Result<()>,
    ) -> Result<()> {
        if let Some((buffer, _)) = &self.input {
            queue.write_buffer(buffer, 0, plan.input_bytes());
        }
        queue.write_buffer(&self.parameters, 0, plan.parameters());
        for (index, draw) in plan.draws().iter().enumerate() {
            check()?;
            // Keep one ordered pass for EVERY draw, including the full clear.
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("one reusable-buffer ordered source-over draw"),
                timestamp_writes: None,
            });
            let (pipeline, group) = if group_dispatch(*draw) {
                self.groups
                    .as_ref()
                    .ok_or("missing group scratch")?
                    .bindings(
                        rasterizer
                            .kernels
                            .groups
                            .as_ref()
                            .ok_or("missing group pipelines")?,
                        draw.kind(),
                    )?
            } else {
                match draw.kind() {
                    DrawKind::Rectangle => (&rasterizer.kernels.rectangle, &self.rectangle_group),
                    DrawKind::Image => (
                        &rasterizer.kernels.image,
                        &self.input.as_ref().ok_or("missing image arena")?.1,
                    ),
                    DrawKind::Glyph => (
                        rasterizer
                            .kernels
                            .glyph
                            .as_ref()
                            .ok_or("missing glyph pipeline")?,
                        &self.input.as_ref().ok_or("missing glyph arena")?.1,
                    ),
                    DrawKind::GroupClear | DrawKind::GroupComposite => {
                        return Err("invalid group dispatch".into());
                    }
                }
            };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[(index * PARAM_STRIDE) as u32]);
            let (x, y) = draw.groups();
            pass.dispatch_workgroups(x, y, 1);
        }
        Ok(())
    }

    pub fn destroy_after_completion(self) {
        self.output.destroy();
        self.parameters.destroy();
        if let Some((buffer, _)) = self.input {
            buffer.destroy();
        }
        if let Some(groups) = self.groups {
            groups.destroy_after_completion();
        }
    }
}

pub(crate) fn descriptor_matches(
    actual_bytes: u64,
    actual_usage: wgpu::BufferUsages,
    expected_bytes: u64,
    expected_usage: wgpu::BufferUsages,
) -> bool {
    actual_bytes == expected_bytes && actual_usage == expected_usage
}

pub(crate) fn buffer_matches(buffer: &wgpu::Buffer, bytes: u64, usage: wgpu::BufferUsages) -> bool {
    descriptor_matches(buffer.size(), buffer.usage(), bytes, usage)
}
