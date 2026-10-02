//! Native frame encoding stays on the existing surface owner. Scope collection
//! and queued-write retirement also run when encoding is cancelled or fails.
use super::control::Payload;
use super::{
    Gpu, Packet, Phase, Result, Shared, Step, TimingSubmission, Trace, VerifiedRoute, bounded,
    record, remaining, transfer, wait_future,
};
use eris_raster_core::surface::{NativeBufferLease, NativeEncoder};
use std::{sync::mpsc, time::Instant};

pub(super) struct Kernels {
    encoder: NativeEncoder,
}
impl Kernels {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, deadline: Instant) -> Result<Self> {
        remaining(deadline)?;
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let oom = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
        let result: Result<Self> = (|| {
            let encoder = NativeEncoder::new(device, queue, || remaining(deadline).map(|_| ()))?;
            Ok(Self { encoder })
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

/// Keep the retired charge until the complete handle graph is actually dropped.
/// The callbacks execute without a Shared lock and cannot allocate a replacement
/// before acknowledgment. This helper also permits ownership tests without a GPU.
fn evict_retired<T>(
    slot: &mut Option<T>,
    bytes: impl FnOnce(&T) -> Result<usize>,
    destroy: impl FnOnce(T),
    acknowledge: impl FnOnce(usize) -> Result<()>,
) -> Result<()> {
    let Some(lease) = slot.as_ref() else {
        return Ok(());
    };
    let retained = bytes(lease)?;
    let lease = slot.take().ok_or("native retired slot disappeared")?;
    destroy(lease);
    acknowledge(retained)
}

impl Gpu {
    pub(super) fn evict_native_buffers(&mut self, shared: &Shared) -> Result<()> {
        evict_retired(
            &mut self.retired_native,
            |lease| {
                usize::try_from(lease.buffer_bytes()).map_err(|_| "native retained size".into())
            },
            NativeBufferLease::destroy_after_completion,
            |bytes| shared.retired_native_dropped(bytes),
        )
    }
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
        let required = self
            .native
            .as_ref()
            .ok_or("native raster pipelines unavailable")?
            .encoder
            .requirements(plan, self.config.format)?;
        let layout = required.layout();
        let planned =
            usize::try_from(required.buffer_bytes()).map_err(|_| "native allocation size")?;
        let readback_bytes = if verify { layout.storage_bytes() } else { 0 };
        if verify
            && reference.as_ref().is_none_or(|pixels| {
                pixels.len() as u64 != u64::from(packet.size.0) * u64::from(packet.size.1)
            })
        {
            return Err("native verification requires a complete CPU reference".into());
        }
        if readback_bytes > self.device.limits().max_buffer_size {
            return Err("native readback exceeds device buffer limit".into());
        }
        if self
            .retired_native
            .as_ref()
            .is_some_and(|lease| !lease.is_compatible(&required))
        {
            self.evict_native_buffers(shared)?;
        }
        remaining(deadline)?;
        let readback_charge =
            usize::try_from(readback_bytes).map_err(|_| "native readback size")?;
        // The old bundle remains charged until checkout commits. A miss has
        // already dropped any incompatible bundle before fresh admission.
        let mut lease = if self.retired_native.is_some() {
            shared.checkout_retired_native(planned, readback_charge)?;
            self.retired_native.take()
        } else {
            shared.reserve_native(planned, readback_charge)?;
            None
        };
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
            if lease.is_none() {
                // Allocation has no queue writes. The full bundle is retained
                // outside this closure before any cancellable encode begins.
                lease = Some(kernels.encoder.allocate(&required, &mut check)?);
            }
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
            let active = lease.as_mut().ok_or("native buffer lease unavailable")?;
            kernels
                .encoder
                .encode(active, plan, self.config.format, encoder, &mut check)?;
            check()?;
            let conversion = lease.as_ref().ok_or("native conversion unavailable")?;
            let extent = wgpu::Extent3d {
                width: packet.size.0,
                height: packet.size.1,
                depth_or_array_layers: 1,
            };
            encoder.copy_buffer_to_texture(
                wgpu::TexelCopyBufferInfo {
                    buffer: conversion.output_buffer()?,
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
        // Full retirement and successful scopes are necessary but insufficient
        // for reuse: presentation, currentness and verification must also finish.
        let reusable = retired && draws_ready && !cancelled && outcome.is_ok();
        // Explicit destruction is allowed only for known-retired or never-used
        // buffers. Unknown queue completion uses ordinary drop/owner release.
        if !reusable
            && (retired || !progress.may_have_writes)
            && let Some(lease) = lease.take()
        {
            lease.destroy_after_completion();
        }
        if retired && let Some(buffer) = &readback {
            buffer.destroy();
        }
        drop(readback);
        record::end(timing);
        outcome?;
        remaining(deadline)?;
        let mut presented = false;
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
            presented = true;
        } else {
            drop(frame);
        }
        if suboptimal {
            self.configured = false;
        }
        if let Some(mut lease) = lease.take() {
            if reusable && presented && !suboptimal {
                if self.retired_native.is_some() {
                    return Err("native retired slot already occupied".into());
                }
                // Fallible core validation precedes the byte transfer. The last
                // locked current/deadline check may still decline reuse.
                lease.mark_reusable_after_completion()?;
                if shared.retain_completed_native(packet, planned)? {
                    self.retired_native = Some(lease); // Infallible handle move.
                } else {
                    lease.destroy_after_completion();
                }
            } else if retired || !progress.may_have_writes {
                lease.destroy_after_completion();
            }
            // An uncertain failure only reaches ordinary drop and owner release.
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
    fn retired_eviction_drops_before_releasing_bytes_or_allocating_replacement() {
        use std::sync::{Arc, Mutex};
        struct Retired {
            ledger: Arc<Mutex<usize>>,
            events: Arc<Mutex<Vec<&'static str>>>,
        }
        impl Drop for Retired {
            fn drop(&mut self) {
                // Destruction can reenter the accounting lock. The original
                // charge must still exist throughout the actual handle drop.
                assert_eq!(*self.ledger.lock().unwrap(), 808);
                self.events.lock().unwrap().push("drop");
            }
        }
        let ledger = Arc::new(Mutex::new(808));
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut slot = Some(Retired {
            ledger: ledger.clone(),
            events: events.clone(),
        });
        evict_retired(
            &mut slot,
            |_| Ok(808),
            drop,
            |bytes| {
                assert_eq!(bytes, 808);
                assert_eq!(*events.lock().unwrap(), vec!["drop"]);
                *ledger.lock().unwrap() = 0;
                events.lock().unwrap().push("uncharge");
                Ok(())
            },
        )
        .unwrap();
        assert!(slot.is_none());
        assert_eq!(*ledger.lock().unwrap(), 0);
        events.lock().unwrap().push("replacement allocation");
        assert_eq!(
            *events.lock().unwrap(),
            vec!["drop", "uncharge", "replacement allocation"]
        );
    }
    #[test]
    fn failed_eviction_keeps_handles_or_conservative_charge_and_blocks_replacement() {
        use std::cell::Cell;
        let destroyed = Cell::new(false);
        let acknowledged = Cell::new(false);
        let mut slot = Some(808usize);
        assert!(
            evict_retired(
                &mut slot,
                |_| Err("size overflow".into()),
                |_| destroyed.set(true),
                |_| {
                    acknowledged.set(true);
                    Ok(())
                }
            )
            .is_err()
        );
        assert_eq!(slot, Some(808));
        assert!(!destroyed.get() && !acknowledged.get());
        let charge = Cell::new(808);
        let result = evict_retired(
            &mut slot,
            |value| Ok(*value),
            |_| destroyed.set(true),
            |_| {
                acknowledged.set(true);
                Err("ledger mismatch".into())
            },
        );
        if result.is_ok() {
            charge.set(0);
        }
        assert!(result.is_err() && slot.is_none());
        assert!(destroyed.get() && acknowledged.get());
        assert_eq!(charge.get(), 808);
    }
    #[test]
    fn empty_retired_slot_does_not_touch_allocation_or_ledger_callbacks() {
        let mut slot: Option<usize> = None;
        evict_retired(
            &mut slot,
            |_| panic!("empty lookup"),
            |_| panic!("empty destroy"),
            |_| panic!("empty acknowledgment"),
        )
        .unwrap();
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
