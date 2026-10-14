//! Flat Promise records and prepaid publication into the realm's single FIFO.
use super::*;
use std::mem::size_of;

mod factories;
mod install;
#[cfg(test)]
mod install_tests;
#[cfg(test)]
mod quota_more_tests;
#[cfg(test)]
mod quota_tests;
mod semantics;
#[cfg(test)]
mod shared_key_tests;
#[cfg(test)]
use factories::object_slot_plan;
#[cfg(test)]
mod layout_tests;

pub(super) const METADATA_OBJECTS: usize = 7;

pub(super) struct Arena<T> {
    pub(super) records: Vec<T>,
    pub(super) admitted_capacity: usize,
}
impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            admitted_capacity: 0,
        }
    }
}
#[derive(Default)]
pub(super) struct State {
    promises: Arena<PromiseRecord>,
    gates: Arena<ResolveGate>,
    callbacks: Arena<CallbackRecord>,
    captures: Arena<CapabilityCapture>,
    pub(super) jobs: Arena<Option<PromiseJob>>,
    intrinsics: Option<Intrinsics>,
}
pub(super) struct Intrinsics {
    pub(super) constructor: Value,
    pub(super) prototype: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PromiseId(pub(super) usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GateId(pub(super) usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CaptureId(pub(super) usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct JobId(pub(super) usize);

pub(super) struct PromiseRecord {
    object_id: usize,
    settlement: Settlement,
    reactions: Arena<ReactionPair>,
    handled: bool,
}
enum Settlement {
    Pending,
    Fulfilled(Value),
    Rejected(Value),
}
#[derive(Clone)]
pub(super) struct Capability {
    pub(super) promise: Value,
    pub(super) resolve: Value,
    pub(super) reject: Value,
}
struct ReactionPair {
    fulfilled: Option<Value>,
    rejected: Option<Value>,
    capability: Capability,
}
struct ResolveGate {
    target: Option<PromiseId>,
}
struct CallbackRecord {
    object_id: usize,
    kind: CallbackKind,
}
#[derive(Clone, Copy)]
enum CallbackKind {
    Resolve(GateId),
    Reject(GateId),
    CapabilityExecutor(CaptureId),
}
struct CapabilityCapture {
    resolve: Value,
    reject: Value,
}
pub(super) enum PromiseJob {
    Reaction {
        handler: Option<Value>,
        capability: Capability,
        argument: Value,
        kind: ReactionKind,
    },
    ResolveThenable {
        target: PromiseId,
        thenable: Value,
        then: Value,
    },
}
#[derive(Clone, Copy)]
pub(super) enum ReactionKind {
    Fulfill,
    Reject,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) enum QueueJob {
    Callback(Value),
    Promise(JobId),
}

fn overflow() -> ScriptError {
    ScriptError::resource("Promise accounting overflow")
}
fn invalid() -> ScriptError {
    ScriptError::resource("invalid private Promise state")
}
fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or_else(overflow)
}
fn mul(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or_else(overflow)
}
fn search_work(count: usize) -> usize {
    8 + 2 * (usize::BITS - count.leading_zeros()) as usize
}

#[derive(Clone, Copy, Debug)]
struct GrowthPlan {
    target: Option<usize>,
    work: usize,
    bytes: usize,
}
fn growth_plan<T>(len: usize, admitted: usize, append: usize) -> Result<GrowthPlan> {
    let width = size_of::<T>();
    if width == 0 || len > admitted {
        return Err(invalid());
    }
    let need = add(len, append)?;
    let maximum = MAX_HEAP / width;
    if need > maximum {
        return Err(invalid());
    }
    if need <= admitted {
        return Ok(GrowthPlan {
            target: None,
            work: 0,
            bytes: 0,
        });
    }
    let power = need.checked_next_power_of_two().ok_or_else(overflow)?;
    let target = mul(admitted, 2)?.max(8).max(power).min(maximum);
    if target < need || target <= admitted {
        return Err(invalid());
    }
    let movement = width.div_ceil(size_of::<Value>());
    Ok(GrowthPlan {
        target: Some(target),
        work: add(8, mul(mul(2, len)?, movement)?)?,
        bytes: add(32, mul(target, width)?)?,
    })
}

struct SettlementPlan {
    reactions: usize,
    payload: GrowthPlan,
    queue: GrowthPlan,
    work: usize,
    bytes: usize,
}
fn job_plan(
    count: usize,
    fixed: usize,
    jobs_len: usize,
    jobs_admitted: usize,
    queue_len: usize,
    queue_admitted: usize,
) -> Result<SettlementPlan> {
    let payload = growth_plan::<Option<PromiseJob>>(jobs_len, jobs_admitted, count)?;
    let queue = growth_plan::<QueueJob>(queue_len, queue_admitted, count)?;
    Ok(SettlementPlan {
        reactions: count,
        payload,
        queue,
        work: add(add(fixed, payload.work)?, queue.work)?,
        bytes: add(payload.bytes, queue.bytes)?,
    })
}
fn settlement_plan(
    reactions: usize,
    jobs_len: usize,
    jobs_admitted: usize,
    queue_len: usize,
    queue_admitted: usize,
) -> Result<SettlementPlan> {
    job_plan(
        reactions,
        add(16, mul(16, reactions)?)?,
        jobs_len,
        jobs_admitted,
        queue_len,
        queue_admitted,
    )
}
// This token cannot be cloned or constructed outside the paid reserve path.
struct AdmittedSettlement {
    plan: SettlementPlan,
}

impl Runtime {
    pub(super) fn promise_id(&mut self, value: &Value) -> Result<Option<PromiseId>> {
        let result = (|| {
            self.work(search_work(self.promises.promises.records.len()))?;
            let Value::Object(object) = value else {
                return Ok(None);
            };
            Ok(self
                .promises
                .promises
                .records
                .binary_search_by_key(object, |record| record.object_id)
                .ok()
                .map(PromiseId))
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn promise_object_id(&mut self, id: PromiseId) -> Result<usize> {
        let result = (|| {
            self.work(8)?;
            self.promises
                .promises
                .records
                .get(id.0)
                .map(|record| record.object_id)
                .ok_or_else(invalid)
        })();
        self.observe_microtask_failure(result)
    }

    fn plan_promise_jobs(&self, count: usize, fixed: usize) -> Result<SettlementPlan> {
        job_plan(
            count,
            fixed,
            self.promises.jobs.records.len(),
            self.promises.jobs.admitted_capacity,
            self.microtasks.queue.len(),
            self.microtasks.admitted_capacity,
        )
    }

    fn reserve_settlement(&mut self, plan: SettlementPlan) -> Result<AdmittedSettlement> {
        if self.microtasks.suspended {
            return Err(ScriptError::resource(
                "microtask dispatch suspended until reload",
            ));
        }
        self.work(plan.work)?;
        self.charge(plan.bytes)?;
        if let Some(target) = plan.payload.target {
            #[cfg(test)]
            allocation_probe::before(allocation_probe::ReserveSite::SettlementPayload)?;
            self.promises
                .jobs
                .records
                .try_reserve_exact(target - self.promises.jobs.records.len())
                .map_err(|_| ScriptError::resource("Promise job allocation failed"))?;
            #[cfg(test)]
            allocation_probe::completed(allocation_probe::ReserveSite::SettlementPayload);
        }
        if let Some(target) = plan.queue.target {
            #[cfg(test)]
            allocation_probe::before(allocation_probe::ReserveSite::SettlementQueue)?;
            self.microtasks
                .queue
                .try_reserve_exact(target - self.microtasks.queue.len())
                .map_err(|_| ScriptError::resource("Promise FIFO allocation failed"))?;
            #[cfg(test)]
            allocation_probe::completed(allocation_probe::ReserveSite::SettlementQueue);
        }
        Ok(AdmittedSettlement { plan })
    }

    fn publish_job_capacity(&mut self, admitted: AdmittedSettlement) {
        if let Some(target) = admitted.plan.payload.target {
            self.promises.jobs.admitted_capacity = target;
        }
        if let Some(target) = admitted.plan.queue.target {
            self.microtasks.admitted_capacity = target;
        }
    }

    fn push_promise_job(&mut self, job: PromiseJob) {
        let id = JobId(self.promises.jobs.records.len());
        self.promises.jobs.records.push(Some(job));
        self.microtasks.queue.push_back(QueueJob::Promise(id));
    }

    fn commit_settlement(
        &mut self,
        id: PromiseId,
        kind: ReactionKind,
        result: Value,
        admitted: AdmittedSettlement,
    ) {
        debug_assert_eq!(
            admitted.plan.reactions,
            self.promises.promises.records[id.0].reactions.records.len()
        );
        self.publish_job_capacity(admitted);
        let reactions = std::mem::take(&mut self.promises.promises.records[id.0].reactions);
        // Admission covered the entire traversal, all clones/releases and both
        // backing stores. No author calls, allocations or debits occur here.
        for pair in reactions.records {
            let handler = match kind {
                ReactionKind::Fulfill => pair.fulfilled,
                ReactionKind::Reject => pair.rejected,
            };
            self.push_promise_job(PromiseJob::Reaction {
                handler,
                capability: pair.capability,
                argument: result.clone(),
                kind,
            });
        }
        self.promises.promises.records[id.0].settlement = match kind {
            ReactionKind::Fulfill => Settlement::Fulfilled(result),
            ReactionKind::Reject => Settlement::Rejected(result),
        };
    }

    pub(super) fn settle_promise(
        &mut self,
        id: PromiseId,
        kind: ReactionKind,
        result: Value,
    ) -> Result<()> {
        let outcome = (|| {
            let Some(record) = self.promises.promises.records.get(id.0) else {
                self.work(8)?;
                return Err(invalid());
            };
            if !matches!(record.settlement, Settlement::Pending) {
                return self.work(8);
            }
            let plan = settlement_plan(
                record.reactions.records.len(),
                self.promises.jobs.records.len(),
                self.promises.jobs.admitted_capacity,
                self.microtasks.queue.len(),
                self.microtasks.admitted_capacity,
            )?;
            let admitted = self.reserve_settlement(plan)?;
            self.commit_settlement(id, kind, result, admitted);
            Ok(())
        })();
        self.observe_microtask_failure(outcome)
    }

    pub(super) fn append_promise_reaction(
        &mut self,
        id: PromiseId,
        fulfilled: Option<&Value>,
        rejected: Option<&Value>,
        capability: &Capability,
    ) -> Result<()> {
        let result = (|| {
            let Some(record) = self.promises.promises.records.get(id.0) else {
                self.work(18)?;
                return Err(invalid());
            };
            if !matches!(record.settlement, Settlement::Pending) {
                self.work(18)?;
                return Err(invalid());
            }
            let plan = growth_plan::<ReactionPair>(
                record.reactions.records.len(),
                record.reactions.admitted_capacity,
                1,
            )?;
            self.work(add(18, plan.work)?)?;
            self.charge(plan.bytes)?;
            let record = &mut self.promises.promises.records[id.0];
            if let Some(target) = plan.target {
                record
                    .reactions
                    .records
                    .try_reserve_exact(target - record.reactions.records.len())
                    .map_err(|_| ScriptError::resource("Promise reaction allocation failed"))?;
            }
            record.reactions.records.push(ReactionPair {
                fulfilled: fulfilled.cloned(),
                rejected: rejected.cloned(),
                capability: capability.clone(),
            });
            if let Some(target) = plan.target {
                record.reactions.admitted_capacity = target;
            }
            record.handled = true;
            Ok(())
        })();
        self.observe_microtask_failure(result)
    }

    // Direct entry retains the exact standalone quota witness. Production's
    // re-read path below shares its same planner/reservation/publication seam.
    #[cfg(test)]
    fn enqueue_settled_reaction(
        &mut self,
        kind: ReactionKind,
        handler: Option<Value>,
        capability: Capability,
        argument: &Value,
    ) -> Result<()> {
        let result = (|| {
            let plan = self.plan_promise_jobs(1, 16)?;
            let admitted = self.reserve_settlement(plan)?;
            self.publish_job_capacity(admitted);
            self.push_promise_job(PromiseJob::Reaction {
                kind,
                handler,
                capability,
                argument: argument.clone(),
            });
            Ok(())
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn register_promise_then(
        &mut self,
        id: PromiseId,
        fulfilled: Option<&Value>,
        rejected: Option<&Value>,
        capability: Capability,
    ) -> Result<()> {
        let result = (|| {
            // NewPromiseCapability may have executed author code. Read the
            // actual current state now; no snapshot survives that constructor.
            self.work(8)?;
            let record = self
                .promises
                .promises
                .records
                .get(id.0)
                .ok_or_else(invalid)?;
            if matches!(record.settlement, Settlement::Pending) {
                return self.append_promise_reaction(id, fulfilled, rejected, &capability);
            }
            let plan = self.plan_promise_jobs(1, 16)?;
            let admitted = self.reserve_settlement(plan)?;
            let (kind, argument, handler) = match &self.promises.promises.records[id.0].settlement {
                Settlement::Fulfilled(value) => {
                    (ReactionKind::Fulfill, value.clone(), fulfilled.cloned())
                }
                Settlement::Rejected(value) => {
                    (ReactionKind::Reject, value.clone(), rejected.cloned())
                }
                Settlement::Pending => unreachable!("no author calls during Promise admission"),
            };
            // The16 includes the result clone and selected-handler handoff;
            // the consumed capability is moved, never cloned on this branch.
            self.publish_job_capacity(admitted);
            self.push_promise_job(PromiseJob::Reaction {
                kind,
                handler,
                capability,
                argument,
            });
            self.promises.promises.records[id.0].handled = true;
            Ok(())
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn enqueue_resolve_thenable(
        &mut self,
        target: PromiseId,
        thenable: &Value,
        then: &Value,
    ) -> Result<()> {
        let result = (|| {
            let plan = self.plan_promise_jobs(1, 18)?;
            let admitted = self.reserve_settlement(plan)?;
            self.publish_job_capacity(admitted);
            self.push_promise_job(PromiseJob::ResolveThenable {
                target,
                thenable: thenable.clone(),
                then: then.clone(),
            });
            Ok(())
        })();
        self.observe_microtask_failure(result)
    }
}

#[cfg(test)]
mod allocation_probe {
    use super::*;
    use std::cell::Cell;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(super) enum ReserveSite {
        SettlementPayload,
        SettlementQueue,
    }
    #[derive(Clone, Copy, Default)]
    pub(super) struct Observation {
        pub(super) payload_attempts: usize,
        pub(super) payload_completed: usize,
        pub(super) queue_attempts: usize,
        pub(super) queue_completed: usize,
    }
    #[derive(Clone, Copy, Default)]
    struct Probe {
        fail: Option<ReserveSite>,
        observation: Observation,
    }
    thread_local! { static PROBE: Cell<Probe> = const { Cell::new(Probe { fail: None,
    observation: Observation { payload_attempts: 0, payload_completed: 0, queue_attempts: 0, queue_completed: 0 } }) }; }
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
            match site {
                ReserveSite::SettlementPayload => state.observation.payload_attempts += 1,
                ReserveSite::SettlementQueue => state.observation.queue_attempts += 1,
            }
            let fail = state.fail == Some(site);
            if fail {
                state.fail = None;
            }
            cell.set(state);
            if fail {
                Err(ScriptError::resource("injected Promise reserve refusal"))
            } else {
                Ok(())
            }
        })
    }
    pub(super) fn completed(site: ReserveSite) {
        PROBE.with(|cell| {
            let mut state = cell.get();
            match site {
                ReserveSite::SettlementPayload => state.observation.payload_completed += 1,
                ReserveSite::SettlementQueue => state.observation.queue_completed += 1,
            }
            cell.set(state);
        });
    }
}

#[cfg(test)]
mod shared_key_identity_tests;
