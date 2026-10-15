//! Thread-scoped reserve failure injection; absent from production builds.
use super::{Result, ScriptError};
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Observation {
    pub(super) attempts: usize,
    pub(super) completed: usize,
}

#[derive(Clone, Copy, Default)]
struct State {
    active: bool,
    fail: bool,
    observation: Observation,
}

thread_local! {
    static STATE: Cell<State> = const { Cell::new(State {
        active: false, fail: false,
        observation: Observation { attempts: 0, completed: 0 },
    }) };
}

pub(super) struct FaultGuard {
    previous: State,
    // A guard must restore the thread that installed it.
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

pub(super) fn fail_once() -> FaultGuard {
    let previous = STATE.replace(State {
        active: true,
        fail: true,
        observation: Observation::default(),
    });
    FaultGuard {
        previous,
        _thread: std::marker::PhantomData,
    }
}

impl FaultGuard {
    pub(super) fn observation(&self) -> Observation {
        STATE.get().observation
    }
}

impl Drop for FaultGuard {
    fn drop(&mut self) {
        STATE.set(self.previous);
    }
}

pub(super) fn before_reserve() -> Result<()> {
    let mut state = STATE.get();
    if !state.active {
        return Ok(());
    }
    state.observation.attempts += 1;
    let fail = state.fail;
    state.fail = false;
    STATE.set(state);
    if fail {
        Err(ScriptError::resource("injected spread reserve failure"))
    } else {
        Ok(())
    }
}

pub(super) fn completed() {
    let mut state = STATE.get();
    if state.active {
        state.observation.completed += 1;
        STATE.set(state);
    }
}
