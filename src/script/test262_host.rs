//! Opt-in, private Test262 completion evidence under the current author budget.
use super::*;
use std::collections::btree_map::Entry;
use std::mem::size_of;
use std::num::NonZeroUsize;

#[cfg(test)]
mod quota_tests;
#[cfg(test)]
mod tests;

const COMPLETION_ERROR: u16 = 0x001;
const PREMATURE: u16 = 0x002;
const QUEUED_ERROR: u16 = 0x004;
const FATAL: u16 = 0x008;
const UNAUTHENTICATED_PRINT: u16 = 0x010;
const INVALID_PRINT: u16 = 0x020;
const DELEGATE_ABRUPT: u16 = 0x040;
const SESSION_VIOLATION: u16 = 0x080;
const BODY_ABRUPT: u16 = 0x100;
const PRINT: &str = "Test262.print";
const DONE: &str = "Test262.done";
const SUCCESS: &str = "Test262:AsyncTestComplete";
const FAILURE: &str = "Test262:AsyncTestFailure:";

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Test262Policy {
    CheckedSync = 0,
    AsyncDone = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Test262Observation {
    pub policy: Test262Policy,
    pub done_calls: u8,
    pub success_prints: u8,
    pub failure_prints: u8,
    pub flags: u16,
    pub quiescent: bool,
    pub audited: bool,
    pub unhandled: u32,
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum Phase {
    Harness,
    Armed,
    Finished,
}

pub(super) struct Session {
    print_id: usize,
    done_id: usize,
    pending_done: Option<Rc<Native>>,
    original_done: Option<usize>,
    active_done_depth: usize,
    unhandled: u32,
    flags: u16,
    done_calls: u8,
    success_prints: u8,
    failure_prints: u8,
    policy: Test262Policy,
    phase: Phase,
    audited: bool,
}

#[derive(Clone, Copy)]
struct DoneToken {
    original: usize,
    prior_depth: usize,
}

#[derive(Clone, Copy, Debug)]
struct ObjectPlan {
    target: Option<usize>,
    work: usize,
    bytes: usize,
}

fn overflow() -> ScriptError {
    ScriptError::resource("Test262 host accounting overflow")
}
fn invalid() -> ScriptError {
    ScriptError::resource("invalid Test262 host session state")
}
fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or_else(overflow)
}
fn mul(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or_else(overflow)
}
fn tree(count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let mut capacity = count.saturating_add(1) / 2;
    let mut height = 1;
    while capacity >= 6 {
        capacity /= 6;
        height += 1;
    }
    (count.min(11 * height), height)
}

fn object_plan(len: usize, capacity: usize) -> Result<ObjectPlan> {
    let width = size_of::<ScriptObject>();
    let need = add(len, 2)?;
    let maximum = MAX_HEAP / width;
    if len > capacity || need > maximum {
        return Err(invalid());
    }
    let records = mul(2, width)?;
    if need <= capacity {
        return Ok(ObjectPlan {
            target: None,
            work: 0,
            bytes: records,
        });
    }
    let power = need.checked_next_power_of_two().ok_or_else(overflow)?;
    let target = mul(capacity, 2)?.max(8).max(power).min(maximum);
    if target < need || target <= capacity {
        return Err(invalid());
    }
    Ok(ObjectPlan {
        target: Some(target),
        work: add(8, mul(mul(2, len)?, width.div_ceil(size_of::<Value>()))?)?,
        bytes: add(records, add(32, mul(target, width)?)?)?,
    })
}

fn authentic(native: &Native, name: &str, id: usize) -> bool {
    native.name == name
        && matches!(native.receiver, Value::Object(found) if found == id)
        && native
            .properties
            .is_some_and(|properties| properties.get() == id)
}
fn increment(value: &mut u8) {
    *value = value.saturating_add(1).min(2);
}

impl Runtime {
    fn test262_quiescent(&self) -> bool {
        self.microtasks.queue.is_empty()
            && !self.microtasks.performing_checkpoint
            && self.microtasks.author_depth == 0
            && !self.microtasks.suspended
            && self.calls == 0
            && self.frames.is_empty()
            && self.eval_depth == 0
    }

    /// A constant-size diagnostic snapshot, never a pass decision or audit.
    pub fn test262_host_observation(&self) -> Option<Test262Observation> {
        self.test262_session
            .as_ref()
            .map(|session| Test262Observation {
                policy: session.policy,
                done_calls: session.done_calls,
                success_prints: session.success_prints,
                failure_prints: session.failure_prints,
                flags: session.flags,
                quiescent: self.test262_quiescent(),
                audited: session.audited,
                unhandled: session.unhandled,
            })
    }

    pub(in crate::script) fn test262_record_fatal(&mut self) {
        if let Some(session) = &mut self.test262_session {
            session.flags |= FATAL;
        }
    }
    pub(in crate::script) fn test262_record_job_error(&mut self) {
        if let Some(session) = &mut self.test262_session {
            session.flags |= QUEUED_ERROR;
        }
    }
    fn test262_violation(&mut self) -> ScriptError {
        if let Some(session) = &mut self.test262_session {
            session.flags |= SESSION_VIOLATION;
        }
        invalid()
    }

    fn test262_metadata(&mut self, display: &str, internal: &str) -> Result<Rc<Native>> {
        let properties =
            self.dom_proto_object(Some(Value::Function(self.function_prototype)), 2)?;
        let display = self.dom_proto_text(display)?;
        let length = self.dom_proto_text("length")?;
        self.dom_proto_property(
            properties,
            length.into(),
            Property::data(Value::Number(1.0), false, false, true),
        )?;
        let name = self.dom_proto_text("name")?;
        self.dom_proto_property(
            properties,
            name.into(),
            Property::data(Value::String(display), false, false, true),
        )?;
        self.work(add(8, internal.len())?)?;
        self.charge(add(add(size_of::<Native>(), 32)?, internal.len())?)?;
        let mut name = String::new();
        name.try_reserve_exact(internal.len())
            .map_err(|_| ScriptError::resource("Test262 native name allocation failed"))?;
        name.push_str(internal);
        let id = NonZeroUsize::new(properties).ok_or_else(invalid)?;
        Ok(Rc::new(Native {
            name,
            receiver: Value::Object(properties),
            properties: Some(id),
        }))
    }

    /// Install private observers without changing the realm's work allowance.
    pub fn enable_test262_host(&mut self, policy: Test262Policy) -> Result<()> {
        self.enable_test262_host_with(policy, |objects, target| {
            objects
                .try_reserve_exact(target - objects.len())
                .map_err(|_| ScriptError::resource("Test262 metadata arena allocation failed"))
        })
    }

    // The same staged body is used by the isolated reserve-refusal fixture.
    fn enable_test262_host_with(
        &mut self,
        policy: Test262Policy,
        reserve: impl FnOnce(&mut Vec<ScriptObject>, usize) -> Result<()>,
    ) -> Result<()> {
        let result = (|| {
            // Guards/publication plus all temporary and retained handle cleanup.
            self.work(32)?;
            if self.test262_session.is_some() || !self.test262_quiescent() {
                return Err(self.test262_violation());
            }
            let plan = object_plan(self.objects.len(), self.objects.capacity())?;
            self.work(plan.work)?;
            self.charge(plan.bytes)?;
            if let Some(target) = plan.target {
                reserve(&mut self.objects, target)?;
                if self.objects.capacity() < target {
                    return Err(invalid());
                }
            }
            // Each old metadata helper retains its staged work/heap admission.
            let print = self.test262_metadata("print", PRINT)?;
            let done = self.test262_metadata("$DONE", DONE)?;
            let print_id = print.properties.ok_or_else(invalid)?.get();
            let done_id = done.properties.ok_or_else(invalid)?.get();
            let (comparisons, height) = tree(self.environments[0].bindings.len());
            let levels = add(height, 1)?;
            self.work(add(add(add(8, comparisons)?, mul(28, levels)?)?, 5)?)?;
            let block = add(
                add(
                    mul(16, add(size_of::<String>(), size_of::<Binding>())?)?,
                    mul(32, size_of::<usize>())?,
                )?,
                64,
            )?;
            self.charge(add(37, mul(levels, block)?)?)?;
            let next_order = self.next_global_order.checked_add(1).ok_or_else(overflow)?;
            let mut key = String::new();
            key.try_reserve_exact(5)
                .map_err(|_| ScriptError::resource("Test262 global key allocation failed"))?;
            key.push_str("print");
            let Entry::Vacant(entry) = self.environments[0].bindings.entry(key) else {
                return Err(invalid());
            };
            entry.insert(Binding {
                value: Value::Native(print),
                accessor: None,
                mutable: true,
                initialized: true,
                strict_immutable: false,
                global_property: true,
                global_order: self.next_global_order,
                enumerable: false,
                deletable: true,
            });
            self.test262_session = Some(Session {
                print_id,
                done_id,
                pending_done: Some(done),
                original_done: None,
                active_done_depth: 0,
                unhandled: 0,
                flags: 0,
                done_calls: 0,
                success_prints: 0,
                failure_prints: 0,
                policy,
                phase: Phase::Harness,
                audited: false,
            });
            self.next_global_order = next_order;
            Ok(())
        })();
        self.observe_microtask_failure(result)
    }

    /// Called only after the adapter authenticated and executed the pinned blob.
    pub fn bind_test262_done_from_harness(&mut self) -> Result<()> {
        let result = (|| {
            self.work(8)?;
            let valid = self.test262_session.as_ref().is_some_and(|session| {
                session.policy == Test262Policy::AsyncDone
                    && matches!(session.phase, Phase::Harness)
                    && session.original_done.is_none()
                    && session.pending_done.is_some()
                    && session.flags == 0
            });
            if !valid || !self.test262_quiescent() {
                return Err(self.test262_violation());
            }
            let (comparisons, _) = tree(self.environments[0].bindings.len());
            self.work(add(comparisons, 8)?)?;
            let session = self.test262_session.as_mut().ok_or_else(invalid)?;
            let Some(binding) = self.environments[0].bindings.get_mut("$DONE") else {
                session.flags |= SESSION_VIOLATION;
                return Err(invalid());
            };
            let original = match binding.value {
                Value::Function(id)
                    if id < self.functions.len()
                        && binding.initialized
                        && binding.global_property
                        && binding.accessor.is_none() =>
                {
                    id
                }
                _ => {
                    session.flags |= SESSION_VIOLATION;
                    return Err(invalid());
                }
            };
            let done = session.pending_done.take().ok_or_else(invalid)?;
            binding.value = Value::Native(done);
            session.original_done = Some(original);
            Ok(())
        })();
        self.observe_microtask_failure(result)
    }

    pub fn arm_test262_body(&mut self) -> Result<()> {
        let result = (|| {
            self.work(16)?;
            let valid = self.test262_session.as_ref().is_some_and(|session| {
                matches!(session.phase, Phase::Harness)
                    && session.flags == 0
                    && session.done_calls == 0
                    && session.success_prints == 0
                    && session.failure_prints == 0
                    && match session.policy {
                        Test262Policy::CheckedSync => session.original_done.is_none(),
                        Test262Policy::AsyncDone => session.original_done.is_some(),
                    }
            });
            if !valid || !self.test262_quiescent() {
                return Err(self.test262_violation());
            }
            self.test262_session.as_mut().ok_or_else(invalid)?.phase = Phase::Armed;
            Ok(())
        })();
        self.observe_microtask_failure(result)
    }

    fn test262_done_begin(&mut self, native: &Native, args: &[Value]) -> Result<DoneToken> {
        let result = (|| {
            self.work(24)?;
            let valid = self.test262_session.as_ref().is_some_and(|session| {
                session.policy == Test262Policy::AsyncDone
                    && authentic(native, DONE, session.done_id)
                    && session.original_done.is_some()
            });
            if !valid {
                return Err(self.test262_violation());
            }
            let session = self.test262_session.as_mut().ok_or_else(invalid)?;
            let next = session
                .active_done_depth
                .checked_add(1)
                .ok_or_else(overflow)?;
            let token = DoneToken {
                original: session.original_done.ok_or_else(invalid)?,
                prior_depth: session.active_done_depth,
            };
            if !matches!(session.phase, Phase::Armed) {
                session.flags |= PREMATURE;
            }
            if args.first().is_some_and(Value::truthy) {
                session.flags |= COMPLETION_ERROR;
            }
            increment(&mut session.done_calls);
            session.active_done_depth = next;
            Ok(token)
        })();
        self.observe_microtask_failure(result)
    }

    fn test262_done_end(&mut self, token: DoneToken, result: &Result<Value>) {
        if let Some(session) = &mut self.test262_session {
            session.active_done_depth = token.prior_depth;
            if let Err(error) = result {
                session.flags |= DELEGATE_ABRUPT;
                if error.is_resource_limit() || error.is_unsupported() {
                    session.flags |= FATAL;
                }
            }
        }
    }

    fn test262_print(&mut self, native: &Native, args: &[Value]) -> Result<Value> {
        let result = (|| {
            self.work(16)?;
            let valid = self
                .test262_session
                .as_ref()
                .is_some_and(|session| authentic(native, PRINT, session.print_id));
            if !valid {
                return Err(self.test262_violation());
            }
            let session = self.test262_session.as_mut().ok_or_else(invalid)?;
            if !matches!(session.phase, Phase::Armed) {
                session.flags |= PREMATURE;
            }
            if session.active_done_depth == 0 {
                session.flags |= UNAUTHENTICATED_PRINT;
                return Ok(Value::Undefined);
            }
            let Some(Value::String(text)) = args.first() else {
                session.flags |= INVALID_PRINT;
                return Ok(Value::Undefined);
            };
            self.work(2 * text.len().min(25))?;
            let success = text.len() == 25
                && text
                    .units()
                    .iter()
                    .copied()
                    .eq(SUCCESS.bytes().map(u16::from));
            let failure = !success
                && text.len() >= 25
                && text.units()[..25]
                    .iter()
                    .copied()
                    .eq(FAILURE.bytes().map(u16::from));
            let session = self.test262_session.as_mut().ok_or_else(invalid)?;
            if success {
                increment(&mut session.success_prints);
            } else if failure {
                increment(&mut session.failure_prints);
                session.flags |= COMPLETION_ERROR;
            } else {
                session.flags |= INVALID_PRINT;
            }
            Ok(Value::Undefined)
        })();
        self.observe_microtask_failure(result)
    }

    pub(in crate::script) fn test262_native_call(
        &mut self,
        native: &Native,
        arguments: Vec<Value>,
        receiver: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        if native.name == PRINT {
            return self.test262_print(native, &arguments);
        }
        let token = self.test262_done_begin(native, &arguments)?;
        // Move the original argument buffer and dynamic receiver into the old
        // call path; no rebuilt args, new turn, coercion or cloned callable.
        let result = self.call(Value::Function(token.original), arguments, receiver, doc);
        self.test262_done_end(token, &result);
        self.observe_microtask_failure(result)
    }

    /// Finish once with the body's remaining budget, never a fresh author turn.
    pub fn finish_test262_body(&mut self, body: &Result<Value>) -> Result<Test262Observation> {
        // Fixed cleanup was admitted by setup, so refusal cannot leave a session
        // retryable or lose an actual body error even with zero remaining work.
        let valid = if let Some(session) = &mut self.test262_session {
            if body.is_err() {
                session.flags |= BODY_ABRUPT;
            }
            let valid = matches!(session.phase, Phase::Armed);
            session.phase = Phase::Finished;
            if !valid {
                session.flags |= SESSION_VIOLATION;
            }
            valid
        } else {
            false
        };
        let result = (|| {
            self.work(16)?;
            if !valid {
                return Err(self.test262_violation());
            }
            if self.test262_quiescent()
                && self
                    .test262_session
                    .as_ref()
                    .is_some_and(|session| session.flags & FATAL == 0)
            {
                let unhandled = self.count_test262_unhandled()?;
                let session = self.test262_session.as_mut().ok_or_else(invalid)?;
                session.unhandled = unhandled;
                session.audited = true;
            }
            self.test262_host_observation().ok_or_else(invalid)
        })();
        self.observe_microtask_failure(result)
    }
}
