//! Native frame encoding stays on the existing surface owner. Scope collection
//! and queued-write retirement also run when encoding is cancelled or fails.
use super::control::Payload;
use super::{
    Gpu, Packet, Phase, Result, Shared, Step, TimingSubmission, Trace, VerifiedRoute, bounded,
    record, remaining, transfer, wait_future,
};
use eris_raster_core::{
    gpu::Rasterizer,
    surface::{ConvertedFrame, SurfaceConverter, SurfaceLayout},
};
use std::{sync::mpsc, time::Instant};

pub(super) struct Kernels {
    rasterizer: Rasterizer,
    converter: SurfaceConverter,
}
impl Kernels {
    pub fn new(device: &wgpu::Device, deadline: Instant) -> Result<Self> {
        remaining(deadline)?;
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let oom = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
        let result: Result<Self> = (|| {
            let rasterizer = Rasterizer::new(device, true);
            remaining(deadline)?;
            let converter = SurfaceConverter::new(device);
            remaining(deadline)?;
            Ok(Self {
                rasterizer,
                converter,
            })
        })();
        let mut status = result.as_ref().map(|_| ()).map_err(Clone::clone);
        collect_scopes([internal, oom, validation], deadline, &mut status);
        status?;
        result
    }
}

fn save_error(outcome: &mut Result<()>, result: Result<()>) {
    if outcome.is_ok() {
        *outcome = result;
    }
}

fn collect_scopes(scopes: [wgpu::ErrorScopeGuard; 3], deadline: Instant, outcome: &mut Result<()>) {
    // Every guard is popped even if an earlier pop/error/deadline failed. Never
    // let a scope error turn into an admission fallback or a stale-frame success.
    for scope in scopes {
        let result = match wait_future(scope.pop(), deadline) {
            Ok(None) => Ok(()),
            Ok(Some(error)) => Err(bounded(format_args!("native raster error scope: {error}"))),
            Err(error) => Err(error),
        };
        save_error(outcome, result);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SubmissionKind {
    Draws,
    Flush,
}
#[derive(Default)]
struct QueueProgress {
    may_have_writes: bool,
    submitted: bool,
}
impl QueueProgress {
    fn before_encode(&mut self) {
        self.may_have_writes = true;
    }
    fn take_submission(&mut self, draws_ready: bool) -> Result<Option<SubmissionKind>> {
        if self.submitted {
            return Err("native frame attempted a second submission".into());
        }
        if draws_ready || self.may_have_writes {
            self.submitted = true;
            Ok(Some(if draws_ready {
                SubmissionKind::Draws
            } else {
                SubmissionKind::Flush
            }))
        } else {
            Ok(None)
        }
    }
}

impl Gpu {
    pub(super) fn frame_native(
        &mut self,
        packet: &Packet,
        shared: &Shared,
        verify: bool,
        timing: &mut Option<Trace>,
    ) -> Result<()> {
        let Payload::Native { plan, reference } = &packet.payload else {
            return Err("native raster requires an admitted plan".into());
        };
        let deadline = shared.deadline()?;
        record::begin(timing, Step::Acquire);
        let acquired = self.acquire(packet, shared, deadline, timing);
        record::end(timing);
        let Some((frame, suboptimal)) = acquired? else {
            return Ok(());
        };
        record::begin(timing, Step::EncodeUpload);
        let layout = SurfaceLayout::for_plan(plan, self.config.format)?;
        let readback_bytes = if verify { layout.storage_bytes() } else { 0 };
        if verify
            && reference.as_ref().is_none_or(|pixels| {
                pixels.len() as u64 != u64::from(packet.size.0) * u64::from(packet.size.1)
            })
        {
            return Err("native verification requires a complete CPU reference".into());
        }
        shared.reserve_native(
            usize::try_from(plan.gpu_buffer_bytes()).map_err(|_| "native allocation size")?,
            usize::try_from(readback_bytes).map_err(|_| "native readback size")?,
        )?;
        if readback_bytes > self.device.limits().max_buffer_size {
            return Err("native readback exceeds device buffer limit".into());
        }
        let kernels = self
            .native
            .as_ref()
            .ok_or("native raster pipelines unavailable")?;
        remaining(deadline)?;
        let validation = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let oom = self.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let internal = self.device.push_error_scope(wgpu::ErrorFilter::Internal);
        let mut progress = QueueProgress::default();
        let mut cancelled = false;
        let mut encoder = None;
        let mut converted: Option<ConvertedFrame> = None;
        let mut readback = None;
        let mut check = || {
            remaining(deadline)?;
            if !shared.current(packet) {
                cancelled = true;
                return Err("native frame became obsolete".into());
            }
            Ok(())
        };
        let encoded: Result<()> = (|| {
            check()?;
            encoder = Some(
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Eris complete native frame"),
                    }),
            );
            if verify {
                readback = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("native acquired surface verification"),
                    size: readback_bytes,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }));
            }
            check()?;
            let encoder = encoder.as_mut().ok_or("native encoder unavailable")?;
            shared.phase(Phase::Encoding);
            // A callback can refuse after core input/uniform writes were queued.
            // Conservatively enter this state before the first helper call.
            progress.before_encode();
            let source =
                kernels
                    .rasterizer
                    .encode(&self.device, &self.queue, plan, encoder, &mut check)?;
            converted = Some(kernels.converter.encode(
                &self.device,
                &self.queue,
                source,
                self.config.format,
                encoder,
                &mut check,
            )?);
            check()?;
            let conversion = converted.as_ref().ok_or("native conversion unavailable")?;
            let extent = wgpu::Extent3d {
                width: packet.size.0,
                height: packet.size.1,
                depth_or_array_layers: 1,
            };
            encoder.copy_buffer_to_texture(
                wgpu::TexelCopyBufferInfo {
                    buffer: conversion.output_buffer(),
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(layout.padded_bytes_per_row()),
                        rows_per_image: Some(packet.size.1),
                    },
                },
                frame.texture.as_image_copy(),
                extent,
            );
            if let Some(buffer) = &readback {
                encoder.copy_texture_to_buffer(
                    frame.texture.as_image_copy(),
                    wgpu::TexelCopyBufferInfo {
                        buffer,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(layout.padded_bytes_per_row()),
                            rows_per_image: Some(packet.size.1),
                        },
                    },
                    extent,
                );
            }
            check()?;
            Ok(())
        })();
        record::end(timing);
        let draws_ready = encoded.is_ok();
        if cancelled && let Some(trace) = timing {
            trace.aborted();
        }
        let mut outcome = if cancelled { Ok(()) } else { encoded };
        // No question-mark escape from here until scope/retirement cleanup.
        let decision = progress.take_submission(draws_ready);
        let has_submission = matches!(&decision, Ok(Some(_)));
        if has_submission {
            record::begin(timing, Step::Submit);
        }
        let submission = match decision {
            Ok(Some(SubmissionKind::Draws)) => match encoder.take() {
                Some(encoder) => {
                    if let Some(trace) = timing {
                        trace.submitted(TimingSubmission::Draws);
                    }
                    Some(self.queue.submit([encoder.finish()]))
                }
                None => {
                    save_error(
                        &mut outcome,
                        Err("native encoder lost before submission".into()),
                    );
                    // Metadata may exist even if an internal invariant failed.
                    if let Some(trace) = timing {
                        trace.submitted(TimingSubmission::Flush);
                    }
                    Some(self.queue.submit([]))
                }
            },
            Ok(Some(SubmissionKind::Flush)) => {
                drop(encoder.take()); // Never execute an incomplete draw prefix.
                if let Some(trace) = timing {
                    trace.submitted(TimingSubmission::Flush);
                    trace.aborted();
                }
                Some(self.queue.submit([]))
            }
            Ok(None) => {
                drop(encoder.take());
                None
            }
            Err(error) => {
                save_error(&mut outcome, Err(error));
                None
            }
        };
        if has_submission {
            record::end(timing);
        }
        let mut retired = false;
        if let Some(submission) = submission {
            shared.phase(Phase::Submitted);
            let mapping = if draws_ready {
                readback.as_ref().map(|buffer| {
                    let (tx, rx) = mpsc::sync_channel(1);
                    buffer.map_async(wgpu::MapMode::Read, .., move |result| {
                        let _ = tx.try_send(result);
                    });
                    rx
                })
            } else {
                None
            };
            shared.phase(Phase::Retiring);
            record::begin(timing, Step::CompletionWait);
            let completion = (|| {
                self.device
                    .poll(wgpu::PollType::Wait {
                        submission_index: Some(submission),
                        timeout: Some(remaining(deadline)?),
                    })
                    .map_err(bounded)?;
                retired = true;
                remaining(deadline)?;
                Ok(())
            })();
            record::end(timing);
            save_error(&mut outcome, completion);
            if outcome.is_ok()
                && let (Some(buffer), Some(mapping)) = (&readback, mapping)
            {
                let mapped_ready = (|| {
                    mapping
                        .recv_timeout(remaining(deadline)?)
                        .map_err(bounded)?
                        .map_err(bounded)
                })();
                match mapped_ready {
                    Ok(()) => {
                        let comparison = (|| {
                            let mapped = buffer.get_mapped_range(..).map_err(bounded)?;
                            let expected =
                                reference.as_ref().ok_or("native reference unavailable")?;
                            transfer::compare_words(
                                &mapped,
                                expected,
                                packet.size,
                                layout.padded_bytes_per_row(),
                                self.config.format,
                            )
                        })();
                        buffer.unmap();
                        save_error(&mut outcome, comparison);
                    }
                    Err(error) => save_error(&mut outcome, Err(error)),
                }
            }
        }
        record::begin(timing, Step::Cleanup);
        collect_scopes([internal, oom, validation], deadline, &mut outcome);
        // Explicit destruction is allowed only after tracked completion. On an
        // unresolved failure ordinary drop precedes whole-owner release instead.
        if retired {
            if let Some(converted) = converted.take() {
                converted.destroy_after_completion();
            }
            if let Some(buffer) = &readback {
                buffer.destroy();
            }
        }
        drop(converted);
        drop(readback);
        record::end(timing);
        outcome?;
        remaining(deadline)?;
        if draws_ready && shared.current(packet) {
            record::begin(timing, Step::Present);
            self.window.pre_present_notify();
            self.queue.present(frame);
            record::end(timing);
            remaining(deadline)?;
            if shared.stopped() {
                return Err("native raster failed during present".into());
            }
            if let Some(trace) = timing {
                trace.presented();
            }
            if timing.is_none() && shared.first_native_presented() {
                eprintln!(
                    "presenter: native shader frame presented; size={}x{} serial={} CPU_upload=false reference={verify}",
                    packet.size.0, packet.size.1, packet.stamp.serial
                );
            }
            if verify {
                let bytes = u64::from(packet.size.0) * u64::from(packet.size.1) * 4;
                shared.verified(packet, VerifiedRoute::NativeRaster, bytes)?;
                eprintln!(
                    "presenter: native shader acquired-texture verified; serial={} generation={} viewport_revision={} size={}x{} compared_bytes={bytes} exact=true (before compositor)",
                    packet.stamp.serial,
                    packet.stamp.generation,
                    packet.stamp.viewport_revision,
                    packet.size.0,
                    packet.size.1
                );
            }
        } else {
            drop(frame);
        }
        if suboptimal {
            self.configured = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoding_abort_flushes_once_but_never_submits_the_draw_prefix() {
        let mut clean = QueueProgress::default();
        assert_eq!(clean.take_submission(false).unwrap(), None);
        let mut cancelled = QueueProgress::default();
        cancelled.before_encode();
        assert_eq!(
            cancelled.take_submission(false).unwrap(),
            Some(SubmissionKind::Flush)
        );
        assert!(cancelled.take_submission(false).is_err());
        assert!(cancelled.take_submission(true).is_err());
        let mut complete = QueueProgress::default();
        complete.before_encode();
        assert_eq!(
            complete.take_submission(true).unwrap(),
            Some(SubmissionKind::Draws)
        );
        assert!(complete.take_submission(false).is_err());
    }
    #[test]
    fn cleanup_keeps_first_failure_and_still_records_a_late_scope_failure() {
        let mut result = Err("encode failure".into());
        save_error(&mut result, Err("scope failure".into()));
        assert_eq!(result.unwrap_err(), "encode failure");
        let mut cancelled = Ok(());
        save_error(
            &mut cancelled,
            Err("scope failure after cancellation".into()),
        );
        assert_eq!(cancelled.unwrap_err(), "scope failure after cancellation");
    }
}
