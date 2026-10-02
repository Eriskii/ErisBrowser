//! Completion-paced native host measurements. All identity work and output are
//! outside the measured preparation span. Ordinary drawing never enters here.
use super::{
    Browser, FrameStamp, Presenter, Target, native_benchmark_identity as identity,
    native_benchmark_report as report, native_scene,
};
use crate::presenter::{Submission, TimingCompletion, TimingOutcome, TimingRequest, TimingRoute};
use std::{io::Write, time::Instant};
use winit::{dpi::PhysicalSize, event::WindowEvent};

const STATUS_TEXT: &str = "Native timing scene";

#[derive(Default)]
pub(super) struct Run {
    scene: Vec<u8>,
    target: Target,
    scale_factor: f64,
    edit_sequence: u64,
    source_load_ms: Option<f64>,
    diagnostics: usize,
    pending: Option<(u8, FrameStamp)>,
    completed: u8,
    checked: bool,
    waiting_size: Option<(u32, u32)>,
    waiting_reports: u8,
}
impl Run {
    pub(super) fn armed(&self) -> bool {
        !self.scene.is_empty()
    }

    fn complete(&mut self, completion: TimingCompletion, requested: u8) -> Result<bool, String> {
        let Some((id, stamp)) = self.pending else {
            return Err("native benchmark completion without an outstanding sample".into());
        };
        if id != completion.id || stamp != completion.stamp || id != self.completed + 1 {
            return Err("native benchmark completion identity mismatch".into());
        }
        if completion.outcome != TimingOutcome::Presented {
            return Err("native benchmark sample was not presented".into());
        }
        if requested == 0 || self.completed >= requested {
            return Err("native benchmark completion exceeds requested count".into());
        }
        self.pending = None;
        self.completed += 1;
        Ok(self.completed == requested)
    }
}

pub(super) fn changes_scene(event: &WindowEvent) -> bool {
    matches!(
        event,
        WindowEvent::Resized(_)
            | WindowEvent::ScaleFactorChanged { .. }
            | WindowEvent::Occluded(_)
            | WindowEvent::KeyboardInput { .. }
            | WindowEvent::ModifiersChanged(_)
            | WindowEvent::Ime(_)
            | WindowEvent::MouseInput { .. }
            | WindowEvent::MouseWheel { .. }
            | WindowEvent::CursorMoved { .. }
            | WindowEvent::Touch(_)
            | WindowEvent::DroppedFile(_)
    )
}

impl Browser {
    fn native_benchmark_enabled(&self) -> bool {
        self.presenter_config.benchmark_frames != 0 || self.presenter_config.benchmark_check
    }

    fn check_native_benchmark_scene(&self) -> Result<(), String> {
        let run = &self.native_benchmark;
        let window = self
            .window
            .as_ref()
            .ok_or("native benchmark window missing")?;
        let size = window.inner_size();
        if !run.armed()
            || self.target != run.target
            || (size.width, size.height) != run.target.size
            || window.scale_factor().to_bits() != run.scale_factor.to_bits()
            || self.edit_sequence != run.edit_sequence
            || self.status != STATUS_TEXT
            || self
                .ready_snapshot
                .value
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_some()
        {
            return Err("native benchmark scene or transport state changed".into());
        }
        // Exact equality, including full image payloads. This pre-touches scene
        // data outside the timing span identically for both routes.
        if identity::capture(self, size)? != run.scene {
            return Err("native benchmark paint inputs changed".into());
        }
        Ok(())
    }

    /// true means this redraw has been handled, including a wait for idle.
    pub(super) fn draw_native_benchmark(
        &mut self,
        size: PhysicalSize<u32>,
    ) -> Result<bool, String> {
        if !self.native_benchmark_enabled() {
            return Ok(false);
        }
        if self.native_benchmark.armed() {
            self.check_native_benchmark_scene()?;
            if self.native_benchmark.pending.is_some() {
                return Ok(true);
            }
        } else {
            let loaded = !self.loading
                && self
                    .snapshot
                    .as_ref()
                    .is_some_and(|s| s.generation == self.generation());
            if !loaded {
                let dimensions = (size.width, size.height);
                if self.native_benchmark.waiting_size != Some(dimensions) {
                    if self.native_benchmark.waiting_reports >= 64 {
                        return Err("native benchmark startup size did not settle".into());
                    }
                    self.native_benchmark.waiting_size = Some(dimensions);
                    self.native_benchmark.waiting_reports += 1;
                    eprintln!(
                        "native benchmark waiting: size={}x{} loaded=false",
                        size.width, size.height
                    );
                }
                // Map the desktop window with the existing startup CPU frame.
                // Check-only verification remains disarmed until scene capture.
                return Ok(false);
            }
        }
        if !self.presenter.as_ref().is_some_and(Presenter::timing_ready) {
            return Ok(true);
        }
        if !self.native_benchmark.armed() {
            if self
                .ready_snapshot
                .value
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_some()
            {
                return Ok(true);
            }
            if self.snapshot.as_ref().is_some_and(|s| {
                url::Url::parse(&s.url).is_ok_and(|u| u.scheme() == "eris" && u.path() == "error")
            }) {
                return Err("native benchmark document load failed".into());
            }
            self.status = STATUS_TEXT.into();
            self.selection.normalize(if self.address_focused {
                &self.address
            } else {
                &self.input_value
            });
            let scene = identity::capture(self, size)?;
            let snapshot = self
                .snapshot
                .as_ref()
                .ok_or("native benchmark snapshot missing")?;
            let scale_factor = self
                .window
                .as_ref()
                .ok_or("native benchmark window missing")?
                .scale_factor();
            if !scale_factor.is_finite() || scale_factor <= 0.0 {
                return Err("native benchmark invalid device scale".into());
            }
            self.native_benchmark.scene = scene;
            self.native_benchmark.target = self.target;
            self.native_benchmark.scale_factor = scale_factor;
            self.native_benchmark.edit_sequence = self.edit_sequence;
            self.native_benchmark.source_load_ms = Some(snapshot.load_ms);
            self.native_benchmark.diagnostics = snapshot.diagnostics.len();
        }
        let check = self.presenter_config.benchmark_check;
        if check {
            self.presenter
                .as_mut()
                .ok_or("native benchmark presenter missing")?
                .arm_benchmark_check()?;
        }
        let id = if check {
            0
        } else {
            self.native_benchmark.completed + 1
        };
        // Lazy construction is essential: check-only runs read no timing clock.
        let timing = (!check).then(|| TimingRequest {
            id,
            preparation_started: Instant::now(),
        });
        let (stamp, submission) = if self.presenter_config.native_raster {
            let prepared = native_scene::prepare(self, size, check)
                .map_err(|reason| format!("native benchmark admission refused: {reason}"))?;
            let reference = if check {
                let canvas = self.paint_canvas(size)?;
                if canvas.exhausted() {
                    return Err("native benchmark CPU reference exhausted".into());
                }
                Some(canvas)
            } else {
                None
            };
            self.selection = prepared.selection;
            let stamp = self.next_frame_stamp()?;
            let presenter = self
                .presenter
                .as_mut()
                .ok_or("native benchmark presenter missing")?;
            let submission = match timing {
                Some(timing) => {
                    presenter.submit_native_timed(prepared.plan.into_plan(), stamp, timing)?
                }
                None => presenter.submit_native(prepared.plan.into_plan(), reference, stamp)?,
            };
            (stamp, submission)
        } else {
            let canvas = self.paint_canvas(size)?;
            if canvas.exhausted() {
                return Err("native benchmark CPU paint exhausted".into());
            }
            let stamp = self.next_frame_stamp()?;
            let presenter = self
                .presenter
                .as_mut()
                .ok_or("native benchmark presenter missing")?;
            let submission = match timing {
                Some(timing) => presenter.submit_timed(canvas, stamp, timing)?,
                None => presenter.submit(canvas, stamp)?,
            };
            (stamp, submission)
        };
        if submission != Submission::Queued {
            return Err("native benchmark frame was not admitted as a new queued packet".into());
        }
        self.native_benchmark.pending = Some((id, stamp));
        Ok(true)
    }

    pub(super) fn service_native_benchmark(
        &mut self,
        completion: Option<TimingCompletion>,
    ) -> Result<(), String> {
        if !self.native_benchmark_enabled() {
            return Ok(());
        }
        if let Some(completion) = completion {
            self.check_native_benchmark_scene()?;
            if self
                .native_benchmark
                .complete(completion, self.presenter_config.benchmark_frames)?
            {
                self.begin_close();
            } else {
                self.redraw();
            }
        } else if self.presenter_config.benchmark_check
            && self.native_benchmark.pending.is_some()
            && self
                .presenter
                .as_ref()
                .is_some_and(Presenter::benchmark_check_complete)
        {
            self.check_native_benchmark_scene()?;
            self.native_benchmark.pending = None;
            self.native_benchmark.completed = 1;
            self.native_benchmark.checked = true;
            self.begin_close();
        } else if !self.closing
            && self.native_benchmark.pending.is_none()
            && !self.loading
            && self.snapshot.is_some()
            && self.presenter.as_ref().is_some_and(Presenter::timing_ready)
        {
            self.redraw();
        }
        Ok(())
    }

    /// Called after finish and worker reaping. Report extraction requires the
    /// presenter's positive release acknowledgement, including on failed runs.
    pub(super) fn finish_native_benchmark(
        &mut self,
        mut result: Result<(), String>,
    ) -> Result<(), String> {
        if !self.native_benchmark_enabled() {
            return result;
        }
        // A timed-out or merely finished thread is not a release receipt. Do
        // not emit even a failed JSON report while surface ownership persists.
        if !self
            .presenter
            .as_ref()
            .is_some_and(Presenter::benchmark_owner_released)
        {
            return result.and(Err(
                "native benchmark owner release is unconfirmed; report withheld".into(),
            ));
        }
        let check = self.presenter_config.benchmark_check;
        let requested = if check {
            1
        } else {
            self.presenter_config.benchmark_frames
        };
        let timing = if check {
            None
        } else {
            match self
                .presenter
                .as_mut()
                .ok_or_else(|| "native benchmark presenter missing".to_owned())
                .and_then(Presenter::take_timing_report)
            {
                Ok(report) => Some(report),
                Err(error) => {
                    if result.is_ok() {
                        result = Err(error);
                    }
                    None
                }
            }
        };
        if result.is_ok()
            && (self.native_benchmark.completed != requested
                || self.native_benchmark.pending.is_some()
                || !self.native_benchmark.armed()
                || (check && !self.native_benchmark.checked))
        {
            result = Err("native benchmark ended before all requested frames completed".into());
        }
        if result.is_ok()
            && let Some(report) = &timing
            && (report.requested != requested
                || report.accepted != requested
                || report.samples.len() != usize::from(requested)
                || report.failure.is_some()
                || report
                    .samples
                    .iter()
                    .any(|sample| sample.outcome != TimingOutcome::Presented))
        {
            result = Err("native benchmark owner report is incomplete or failed".into());
        }
        let run = &self.native_benchmark;
        let mut output = std::io::stdout().lock();
        let written = report::write(
            &mut output,
            report::Context {
                check,
                route: if self.presenter_config.native_raster {
                    TimingRoute::NativeRaster
                } else {
                    TimingRoute::CpuUpload
                },
                requested,
                completed: run.completed,
                scene: &run.scene,
                target: if run.armed() { run.target } else { self.target },
                scale_factor: if run.armed() { run.scale_factor } else { 1.0 },
                source_load_ms: run.source_load_ms,
                edit_sequence: run.edit_sequence,
                diagnostics: run.diagnostics,
                checked: run.checked,
                failure: result.as_ref().err().map(String::as_str),
            },
            timing.as_ref(),
        )
        .and_then(|()| output.flush().map_err(|e| e.to_string()));
        result.and(written)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_requires_exact_outstanding_identity_and_positive_present() {
        let stamp = FrameStamp {
            generation: 4,
            viewport_revision: 3,
            serial: 9,
        };
        let mut run = Run {
            pending: Some((1, stamp)),
            ..Run::default()
        };
        let mut completion = TimingCompletion {
            id: 1,
            stamp,
            outcome: TimingOutcome::Presented,
        };
        completion.stamp.serial += 1;
        assert!(run.complete(completion, 2).is_err());
        completion.stamp = stamp;
        for outcome in [
            TimingOutcome::NotPresented,
            TimingOutcome::Aborted,
            TimingOutcome::Failed,
        ] {
            completion.outcome = outcome;
            assert!(run.complete(completion, 2).is_err());
            assert_eq!(run.completed, 0);
            assert_eq!(run.pending, Some((1, stamp)));
        }
        completion.outcome = TimingOutcome::Presented;
        assert!(!run.complete(completion, 2).unwrap());
        assert!(run.complete(completion, 2).is_err());
        run.pending = Some((
            2,
            FrameStamp {
                serial: 10,
                ..stamp
            },
        ));
        assert!(
            run.complete(
                TimingCompletion {
                    id: 2,
                    stamp: run.pending.unwrap().1,
                    outcome: TimingOutcome::Presented
                },
                2
            )
            .unwrap()
        );
        assert_eq!(run.completed, 2);
    }

    #[test]
    fn check_mode_cannot_be_satisfied_by_a_timing_completion() {
        let stamp = FrameStamp::default();
        let mut run = Run {
            pending: Some((0, stamp)),
            ..Run::default()
        };
        assert!(
            run.complete(
                TimingCompletion {
                    id: 0,
                    stamp,
                    outcome: TimingOutcome::Presented
                },
                0
            )
            .is_err()
        );
        assert!(!run.checked);
    }

    #[test]
    fn disabled_benchmark_preserves_ui_and_does_not_require_a_presenter() {
        let mut browser = super::super::tests::editing_browser("<p>A</p>");
        browser.status = "normal status".into();
        assert!(
            !browser
                .draw_native_benchmark(PhysicalSize::new(1180, 880))
                .unwrap()
        );
        browser.service_native_benchmark(None).unwrap();
        assert_eq!(browser.status, "normal status");
        assert_eq!(browser.native_benchmark.scene.capacity(), 0);
        assert!(browser.finish_native_benchmark(Ok(())).is_ok());
    }

    #[test]
    fn incidental_redraw_is_not_a_scene_change_but_resize_and_occlusion_are() {
        assert!(!changes_scene(&WindowEvent::RedrawRequested));
        assert!(!changes_scene(&WindowEvent::Focused(true)));
        assert!(changes_scene(&WindowEvent::Resized(PhysicalSize::new(
            1280, 880
        ))));
        assert!(changes_scene(&WindowEvent::Occluded(true)));
        assert!(changes_scene(&WindowEvent::Ime(winit::event::Ime::Commit(
            "x".into()
        ))));
    }
}
