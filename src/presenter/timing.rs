//! Optional bounded host-clock observations. These are not GPU timestamps.
// The uniform facade exists in other builds, but configuration forbids timing.
#![cfg_attr(
    not(all(target_os = "linux", feature = "vulkan-raster")),
    allow(dead_code)
)]
use super::FrameStamp;
use std::time::Instant;

pub(crate) const MAX_SAMPLES: usize = 128;
pub(crate) const IDENTITY_RESERVE: usize = 256 * 1024;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TimingRequest {
    pub id: u8,
    pub preparation_started: Instant,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TimingRoute {
    CpuUpload,
    NativeRaster,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TimingOutcome {
    Presented,
    NotPresented,
    Aborted,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TimingSubmission {
    None,
    Draws,
    Flush,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct TimingSample {
    pub id: u8,
    pub stamp: FrameStamp,
    pub size: (u32, u32),
    pub route: TimingRoute,
    pub outcome: TimingOutcome,
    pub submission: TimingSubmission,
    pub configured: bool,
    pub ui_prepare_ns: u64,
    pub queue_ns: u64,
    pub acquire_ns: Option<u64>,
    pub encode_upload_ns: Option<u64>,
    pub submit_ns: Option<u64>,
    pub completion_wait_ns: Option<u64>,
    pub cleanup_ns: Option<u64>,
    pub present_call_ns: Option<u64>,
    pub owner_total_ns: u64,
    pub prepare_to_present_ns: Option<u64>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct TimingCompletion {
    pub id: u8,
    pub stamp: FrameStamp,
    pub outcome: TimingOutcome,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct TimingInit {
    pub owner_init_ns: u64,
}
#[derive(Debug)]
pub(crate) struct TimingReport {
    pub requested: u8,
    pub accepted: u8,
    pub initialization: Option<TimingInit>,
    pub samples: Vec<TimingSample>,
    pub failure: Option<String>,
}

#[cfg(all(target_os = "linux", feature = "vulkan-presenter"))]
pub(super) mod record {
    use super::*;

    #[derive(Clone, Copy)]
    pub(in crate::presenter) struct PacketTiming {
        pub request: TimingRequest,
        pub accepted: Option<Instant>,
    }
    #[derive(Clone, Copy, Debug)]
    pub(in crate::presenter) enum Step {
        Acquire,
        EncodeUpload,
        Submit,
        CompletionWait,
        Cleanup,
        Present,
    }
    pub(in crate::presenter) struct Trace {
        request: TimingRequest,
        taken: Instant,
        step: Option<(Step, Instant)>,
        sample: TimingSample,
        invalid: bool,
    }
    fn ns(start: Instant, end: Instant) -> Option<u64> {
        u64::try_from(end.checked_duration_since(start)?.as_nanos()).ok()
    }
    impl Trace {
        pub fn new(
            tag: PacketTiming,
            stamp: FrameStamp,
            size: (u32, u32),
            route: TimingRoute,
        ) -> Self {
            Self::new_at(tag, stamp, size, route, Instant::now())
        }
        fn new_at(
            tag: PacketTiming,
            stamp: FrameStamp,
            size: (u32, u32),
            route: TimingRoute,
            now: Instant,
        ) -> Self {
            let accepted = tag.accepted.unwrap_or(now);
            let prep = ns(tag.request.preparation_started, accepted);
            let queued = ns(accepted, now);
            Self {
                request: tag.request,
                taken: now,
                step: None,
                invalid: tag.accepted.is_none() || prep.is_none() || queued.is_none(),
                sample: TimingSample {
                    id: tag.request.id,
                    stamp,
                    size,
                    route,
                    outcome: TimingOutcome::NotPresented,
                    submission: TimingSubmission::None,
                    configured: false,
                    ui_prepare_ns: prep.unwrap_or(0),
                    queue_ns: queued.unwrap_or(0),
                    acquire_ns: None,
                    encode_upload_ns: None,
                    submit_ns: None,
                    completion_wait_ns: None,
                    cleanup_ns: None,
                    present_call_ns: None,
                    owner_total_ns: 0,
                    prepare_to_present_ns: None,
                },
            }
        }
        fn begin_at(&mut self, step: Step, now: Instant) {
            if self.step.is_some() {
                self.invalid = true;
            }
            self.step = Some((step, now));
        }
        fn end_at(&mut self, now: Instant) {
            let Some((step, start)) = self.step.take() else {
                self.invalid = true;
                return;
            };
            let value = ns(start, now);
            if value.is_none() {
                self.invalid = true;
            }
            let slot = match step {
                Step::Acquire => &mut self.sample.acquire_ns,
                Step::EncodeUpload => &mut self.sample.encode_upload_ns,
                Step::Submit => &mut self.sample.submit_ns,
                Step::CompletionWait => &mut self.sample.completion_wait_ns,
                Step::Cleanup => &mut self.sample.cleanup_ns,
                Step::Present => &mut self.sample.present_call_ns,
            };
            if slot.is_some() {
                self.invalid = true;
            }
            *slot = value;
        }
        pub fn configured(&mut self) {
            self.sample.configured = true;
        }
        pub fn submitted(&mut self, kind: TimingSubmission) {
            self.sample.submission = kind;
        }
        pub fn aborted(&mut self) {
            self.sample.outcome = TimingOutcome::Aborted;
        }
        pub fn presented(&mut self) {
            self.sample.outcome = TimingOutcome::Presented;
            self.sample.prepare_to_present_ns =
                ns(self.request.preparation_started, Instant::now());
            self.invalid |= self.sample.prepare_to_present_ns.is_none();
        }
        pub fn finish(self, failed: bool) -> Result<TimingSample, String> {
            self.finish_at(failed, Instant::now())
        }
        fn finish_at(mut self, failed: bool, now: Instant) -> Result<TimingSample, String> {
            // A failed graphics call may leave its phase open. Preserve its
            // reached duration, but never turn it into a presented sample.
            if self.step.is_some() {
                self.end_at(now);
            }
            self.sample.owner_total_ns = ns(self.taken, now).ok_or("timing clock order")?;
            if failed {
                self.sample.outcome = TimingOutcome::Failed;
            }
            if self.invalid {
                return Err("invalid native timing sequence".into());
            }
            Ok(self.sample)
        }
    }
    fn with_clock(
        trace: &mut Option<Trace>,
        clock: impl FnOnce() -> Instant,
        f: impl FnOnce(&mut Trace, Instant),
    ) {
        if let Some(trace) = trace {
            f(trace, clock());
        }
    }
    pub(in crate::presenter) fn begin(trace: &mut Option<Trace>, step: Step) {
        with_clock(trace, Instant::now, |trace, now| trace.begin_at(step, now));
    }
    pub(in crate::presenter) fn end(trace: &mut Option<Trace>) {
        with_clock(trace, Instant::now, Trace::end_at);
    }
    pub(in crate::presenter) fn initialization(start: Instant) -> Result<TimingInit, String> {
        Ok(TimingInit {
            owner_init_ns: ns(start, Instant::now()).ok_or("timing initialization clock order")?,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::{cell::Cell, time::Duration};
        #[test]
        fn disabled_trace_never_reads_the_optional_clock() {
            let calls = Cell::new(0);
            with_clock(
                &mut None,
                || {
                    calls.set(calls.get() + 1);
                    Instant::now()
                },
                |_, _| panic!("disabled trace"),
            );
            assert_eq!(calls.get(), 0);
        }
        #[test]
        fn synthetic_intervals_keep_route_queue_and_unreached_phases_distinct() {
            let start = Instant::now();
            let tag = PacketTiming {
                request: TimingRequest {
                    id: 1,
                    preparation_started: start,
                },
                accepted: Some(start + Duration::from_nanos(10)),
            };
            let mut trace = Trace::new_at(
                tag,
                FrameStamp::default(),
                (2, 3),
                TimingRoute::NativeRaster,
                start + Duration::from_nanos(15),
            );
            trace.begin_at(Step::Acquire, start + Duration::from_nanos(17));
            trace.end_at(start + Duration::from_nanos(20));
            trace.submitted(TimingSubmission::Flush);
            trace.aborted();
            let sample = trace
                .finish_at(false, start + Duration::from_nanos(30))
                .unwrap();
            assert_eq!(sample.ui_prepare_ns, 10);
            assert_eq!(sample.queue_ns, 5);
            assert_eq!(sample.acquire_ns, Some(3));
            assert_eq!(sample.owner_total_ns, 15);
            assert_eq!(sample.outcome, TimingOutcome::Aborted);
            assert_eq!(sample.prepare_to_present_ns, None);
            assert_eq!(sample.present_call_ns, None);
        }
        #[test]
        fn late_failure_and_clock_inversion_cannot_be_success() {
            let start = Instant::now();
            let tag = PacketTiming {
                request: TimingRequest {
                    id: 1,
                    preparation_started: start,
                },
                accepted: Some(start),
            };
            let mut trace = Trace::new_at(
                tag,
                FrameStamp::default(),
                (1, 1),
                TimingRoute::CpuUpload,
                start,
            );
            trace.sample.outcome = TimingOutcome::Presented;
            assert_eq!(
                trace.finish_at(true, start).unwrap().outcome,
                TimingOutcome::Failed
            );
            let future = PacketTiming {
                request: TimingRequest {
                    id: 1,
                    preparation_started: start + Duration::from_secs(1),
                },
                ..tag
            };
            assert!(
                Trace::new_at(
                    future,
                    FrameStamp::default(),
                    (1, 1),
                    TimingRoute::CpuUpload,
                    start
                )
                .finish_at(false, start)
                .is_err()
            );
        }
    }
}
