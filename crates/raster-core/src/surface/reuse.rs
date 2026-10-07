//! Additive Native-only exact-size buffer reuse. Existing encoders stay one-shot.
use super::{SurfaceConverter, SurfaceLayout, UNIFORM_BYTES};
use crate::{
    Plan, Profile, Result,
    gpu::{RasterBuffers, RasterRequirements, Rasterizer, buffer_matches},
};
use std::sync::Arc;

struct NativeContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    rasterizer: Rasterizer,
    converter: SurfaceConverter,
}

/// One device/queue/pipeline owner for reusable Native frame resources.
///
/// `device` and `queue` must be the pair from one request_device. Public wgpu
/// does not expose a queue's owning device. The caller's encoder must also come
/// from that device. Cloned device handles or matching adapter descriptions do
/// not authorize a different NativeEncoder to reuse this owner's leases.
/// Caller error scopes, submission, polling and deadlines remain mandatory.
pub struct NativeEncoder {
    context: Arc<NativeContext>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Key {
    raster: RasterRequirements,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    padded_row: u32,
    converted: u64,
}
impl Key {
    fn bytes(self) -> Result<u64> {
        self.raster
            .output
            .checked_add(self.raster.parameters)
            .and_then(|n| n.checked_add(self.raster.input))
            .and_then(|n| n.checked_add(self.raster.scratch))
            .and_then(|n| n.checked_add(self.converted))
            .and_then(|n| n.checked_add(UNIFORM_BYTES))
            .filter(|n| *n <= Profile::Native.max_gpu_buffer_bytes())
            .ok_or_else(|| "native reusable buffer accounting mismatch".into())
    }
}

fn same_requirements<T>(owner: &Arc<T>, key: Key, required_owner: &Arc<T>, required: Key) -> bool {
    Arc::ptr_eq(owner, required_owner) && key == required
}

#[derive(Clone, Copy, Debug)]
struct Preparation {
    key: Key,
    layout: SurfaceLayout,
    bytes: u64,
}
impl Preparation {
    fn checked(plan: &Plan, format: wgpu::TextureFormat, limits: &wgpu::Limits) -> Result<Self> {
        let layout = SurfaceLayout::for_plan(plan, format)?;
        layout.check_device(limits)?;
        let raster = RasterRequirements::for_plan(plan, limits, true, true)?;
        let key = Key {
            raster,
            width: plan.frame().width,
            height: plan.frame().height,
            format,
            padded_row: layout.padded_bytes_per_row(),
            converted: layout.storage_bytes(),
        };
        let bytes = key.bytes()?;
        if bytes != plan.gpu_buffer_bytes() {
            return Err("native reusable buffer accounting mismatch".into());
        }
        Ok(Self { key, layout, bytes })
    }
}

/// Allocation-free checked metadata; private owner identity prevents mixing
/// requirements and leases from different devices or NativeEncoder instances.
#[derive(Clone)]
pub struct NativeRequirements {
    context: Arc<NativeContext>,
    preparation: Preparation,
}
impl NativeRequirements {
    /// Exact descriptor bytes of four buffers, plus optional inputs and scratch.
    /// Excludes readback, driver/bind-group overhead and queue staging storage.
    pub fn buffer_bytes(&self) -> u64 {
        self.preparation.bytes
    }
    pub fn layout(&self) -> SurfaceLayout {
        self.preparation.layout
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lifecycle {
    Fresh,
    EncodingFailed,
    Encoded,
    Reusable,
}
impl Lifecycle {
    fn compatible(self) -> bool {
        matches!(self, Self::Fresh | Self::Reusable)
    }
    fn begin(&mut self) -> Result<()> {
        let ready = self.compatible();
        // Even a rejected repeat poisons the lease; it cannot erase uncertainty.
        *self = Self::EncodingFailed;
        if ready {
            Ok(())
        } else {
            Err("native lease requires unused or retired buffers".into())
        }
    }
    fn encoded(&mut self) {
        *self = Self::Encoded;
    }
    fn reusable(&mut self) -> Result<()> {
        if *self != Self::Encoded {
            return Err("only a completely encoded native lease can be retired".into());
        }
        *self = Self::Reusable;
        Ok(())
    }
    fn output(self) -> Result<()> {
        if self == Self::Encoded {
            Ok(())
        } else {
            Err("native lease has no completely encoded output".into())
        }
    }
}

/// Owns the entire Native raster/conversion resource graph, without Clone.
///
/// Encoding borrows this lease: every error leaves its handles with the caller
/// for queued-write retirement. No Drop implementation explicitly destroys
/// buffers. A shared output reference is for caller reads/copies only; callers
/// must not destroy it or retain additional uses across retirement/reuse.
pub struct NativeBufferLease {
    context: Arc<NativeContext>,
    preparation: Preparation,
    raster: RasterBuffers,
    output: wgpu::Buffer,
    parameters: wgpu::Buffer,
    group: wgpu::BindGroup,
    lifecycle: Lifecycle,
    encoded_layout: Option<SurfaceLayout>,
}
impl NativeBufferLease {
    fn descriptors_match(&self, required: &NativeRequirements) -> bool {
        same_requirements(
            &self.context,
            self.preparation.key,
            &required.context,
            required.preparation.key,
        ) && self.raster.matches(required.preparation.key.raster)
            && buffer_matches(
                &self.output,
                required.preparation.key.converted,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            )
            && buffer_matches(
                &self.parameters,
                UNIFORM_BYTES,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            )
    }

    /// Requires a usable lifecycle, exact owner/key, and actual size/usage
    /// descriptors. Descriptor equality alone does not prove GPU retirement,
    /// allocation success or the absence of external misuse of a shared handle.
    pub fn is_compatible(&self, required: &NativeRequirements) -> bool {
        self.lifecycle.compatible() && self.descriptors_match(required)
    }
    pub fn buffer_bytes(&self) -> u64 {
        self.preparation.bytes
    }
    pub fn output_buffer(&self) -> Result<&wgpu::Buffer> {
        self.lifecycle.output()?;
        Ok(&self.output)
    }
    /// The current frame's checked metadata, never the previous frame's layout.
    pub fn layout(&self) -> Result<SurfaceLayout> {
        self.lifecycle.output()?;
        self.encoded_layout
            .ok_or_else(|| "native lease has no current layout".into())
    }
    /// Caller must first prove completion of this lease's exact submission and
    /// ALL other uses, collect all scopes successfully, and resolve its complete
    /// frame/presentation policy. This does not poll, wait or collect scopes.
    /// Only the owner branch with that proof may return this lease to its cache.
    pub fn mark_reusable_after_completion(&mut self) -> Result<()> {
        self.lifecycle.reusable()?;
        self.encoded_layout = None;
        Ok(())
    }
    /// Explicitly destroy only unused Fresh buffers or AFTER caller-proved
    /// completion of all encoded/submitted/queued uses. On unresolved failure
    /// retain ownership through teardown and use ordinary drop instead.
    pub fn destroy_after_completion(self) {
        self.output.destroy();
        self.parameters.destroy();
        self.raster.destroy_after_completion();
    }
}

impl NativeEncoder {
    /// Establish three caller error scopes first. Checks bracket the existing
    /// pipeline constructors; synchronous driver calls cannot be interrupted.
    /// An error creates no frame commands or queued writes; scopes still need
    /// collection. Device and queue must be the original matching pair.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mut check: impl FnMut() -> Result<()>,
    ) -> Result<Self> {
        check()?;
        let rasterizer = Rasterizer::new_native(device, true);
        check()?;
        let converter = SurfaceConverter::new(device);
        check()?;
        Ok(Self {
            context: Arc::new(NativeContext {
                device: device.clone(),
                queue: queue.clone(),
                rasterizer,
                converter,
            }),
        })
    }

    /// Pure checked preflight: no buffer allocation, queue upload or encoding.
    pub fn requirements(
        &self,
        plan: &Plan,
        format: wgpu::TextureFormat,
    ) -> Result<NativeRequirements> {
        Ok(NativeRequirements {
            context: Arc::clone(&self.context),
            preparation: Preparation::checked(plan, format, &self.context.device.limits())?,
        })
    }

    /// Allocate the entire exact-size set after caller preflight succeeds.
    /// Every Rust Result guard precedes the first resource creation; asynchronous
    /// driver errors remain in caller scopes while this returned lease owns all
    /// handles. No queue write or command is issued by allocation.
    pub fn allocate(
        &self,
        required: &NativeRequirements,
        mut check: impl FnMut() -> Result<()>,
    ) -> Result<NativeBufferLease> {
        if !Arc::ptr_eq(&self.context, &required.context) {
            return Err("native requirements belong to another encoder".into());
        }
        check()?;
        let device = &self.context.device;
        let raster = RasterBuffers::allocate(
            device,
            &self.context.rasterizer,
            required.preparation.key.raster,
        );
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reusable opaque padded surface pixels"),
            size: required.preparation.key.converted,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let parameters = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reusable surface conversion metadata"),
            size: UNIFORM_BYTES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("reusable surface conversion bindings"),
            layout: &self.context.converter.kernel.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: raster.output().as_entire_binding(),
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
        Ok(NativeBufferLease {
            context: Arc::clone(&self.context),
            preparation: required.preparation,
            raster,
            output,
            parameters,
            group,
            lifecycle: Lifecycle::Fresh,
            encoded_layout: None,
        })
    }

    /// Rewrite every input/uniform byte, then encode every ordered draw and
    /// conversion. Err retains the lease but poisons it for reuse. Discard the
    /// partial encoder and retire any queued writes before destruction/teardown.
    /// The supplied encoder must come from this NativeEncoder's device.
    pub fn encode(
        &self,
        lease: &mut NativeBufferLease,
        plan: &Plan,
        format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        mut check: impl FnMut() -> Result<()>,
    ) -> Result<()> {
        lease.encoded_layout = None;
        lease.lifecycle.begin()?;
        check()?;
        let required = self.requirements(plan, format)?;
        if !lease.descriptors_match(&required) {
            return Err("native lease owner or exact buffer descriptors differ".into());
        }
        lease.raster.encode(
            &self.context.rasterizer,
            &self.context.queue,
            plan,
            encoder,
            &mut check,
        )?;
        check()?;
        self.context.queue.write_buffer(
            &lease.parameters,
            0,
            &required.preparation.layout.uniform(),
        );
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("reusable opaque surface conversion"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.context.converter.kernel.pipeline);
            pass.set_bind_group(0, &lease.group, &[]);
            let (x, y) = required.preparation.layout.groups();
            pass.dispatch_workgroups(x, y, 1);
        }
        lease.encoded_layout = Some(required.preparation.layout);
        lease.lifecycle.encoded();
        Ok(())
    }
}

#[cfg(test)]
#[path = "reuse_tests.rs"]
mod tests;
