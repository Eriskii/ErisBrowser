//! Scoped test-only allocation faults; this module has no production footprint.
use super::*;
use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ReserveSite {
    AllCapture,
    Callbacks,
    Objects,
    ArrayValues,
    ArrayProperties,
    ArrayHoles,
    ArrayLengths,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Observation {
    pub(super) attempts: [usize; 7],
    pub(super) completed: [usize; 7],
}
#[derive(Clone, Copy, Default)]
struct Probe {
    fail: Option<ReserveSite>,
    observation: Observation,
}
thread_local! {
    static PROBE: Cell<Probe> = const { Cell::new(Probe {
        fail: None,
        observation: Observation { attempts: [0; 7], completed: [0; 7] },
    }) };
}
pub(super) struct FaultGuard {
    previous: Probe,
}
pub(super) fn fail_once(site: ReserveSite) -> FaultGuard {
    let previous = PROBE.with(|cell| {
        cell.replace(Probe {
            fail: Some(site),
            observation: Observation::default(),
        })
    });
    FaultGuard { previous }
}
impl FaultGuard {
    pub(super) fn observation(&self) -> Observation {
        PROBE.with(|cell| cell.get().observation)
    }
}
impl Drop for FaultGuard {
    fn drop(&mut self) {
        PROBE.with(|cell| cell.set(self.previous));
    }
}
pub(super) fn before(site: ReserveSite) -> Result<()> {
    PROBE.with(|cell| {
        let mut state = cell.get();
        state.observation.attempts[site as usize] += 1;
        let fail = state.fail == Some(site);
        if fail {
            state.fail = None;
        }
        cell.set(state);
        if fail {
            Err(ScriptError::resource(
                "injected Promise.all reserve refusal",
            ))
        } else {
            Ok(())
        }
    })
}
pub(super) fn completed(site: ReserveSite) {
    PROBE.with(|cell| {
        let mut state = cell.get();
        state.observation.completed[site as usize] += 1;
        cell.set(state);
    });
}
