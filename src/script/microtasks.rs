//! One realm-owned FIFO, sharing the current author turn's work and heap limits.
use super::*;
use std::collections::{VecDeque, btree_map::Entry};
use std::fmt::Write;

pub(super) const METADATA_OBJECTS: usize = 1;
const NAME: &str = "queueMicrotask";
const REPORT_PREFIX: &str = "Uncaught microtask exception: ";

#[derive(Default)]
pub(super) struct State {
    pub(super) queue: VecDeque<Value>,
    // Admission is independent of allocator rounding and retained spare space.
    pub(super) admitted_capacity: usize,
    pub(super) author_depth: usize,
    pub(super) performing_checkpoint: bool,
    pub(super) suspended: bool,
}

#[derive(Debug)]
pub(super) struct ReportPlan {
    work: usize,
    bytes: usize,
    rendered_bytes: Option<usize>,
    target_capacity: Option<usize>,
}

fn overflow() -> ScriptError {
    ScriptError::resource("microtask accounting overflow")
}

pub(super) fn report_plan(
    message_bytes: usize,
    offset: Option<usize>,
    console_len: usize,
    console_capacity: usize,
) -> Result<ReportPlan> {
    // Other console producers have their own thresholds. Do not format a
    // suppressed asynchronous report, including its decimal offset.
    if console_len >= 1024 {
        return Ok(ReportPlan {
            work: 8,
            bytes: 0,
            rendered_bytes: None,
            target_capacity: None,
        });
    }
    let mut rendered = REPORT_PREFIX
        .len()
        .checked_add(message_bytes)
        .ok_or_else(overflow)?;
    if let Some(offset) = offset {
        let digits = if offset == 0 {
            1
        } else {
            offset.ilog10() as usize + 1
        };
        rendered = rendered.checked_add(9 + digits).ok_or_else(overflow)?;
    }
    let mut work = rendered
        .checked_mul(2)
        .and_then(|n| n.checked_add(8))
        .ok_or_else(overflow)?;
    let mut bytes = rendered.checked_add(32).ok_or_else(overflow)?;
    let target_capacity = if console_len == console_capacity {
        let target = console_capacity.checked_mul(2).ok_or_else(overflow)?.max(4);
        work = console_len
            .checked_mul(2)
            .and_then(|n| n.checked_add(8))
            .and_then(|n| work.checked_add(n))
            .ok_or_else(overflow)?;
        bytes = target
            .checked_mul(std::mem::size_of::<String>())
            .and_then(|n| n.checked_add(32))
            .and_then(|n| bytes.checked_add(n))
            .ok_or_else(overflow)?;
        Some(target)
    } else {
        None
    };
    Ok(ReportPlan {
        work,
        bytes,
        rendered_bytes: Some(rendered),
        target_capacity,
    })
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

impl Runtime {
    pub(super) fn install_microtask_intrinsic(&mut self) -> Result<()> {
        let properties =
            self.dom_proto_object(Some(Value::Function(self.function_prototype)), 2)?;
        let display = self.dom_proto_text(NAME)?;
        self.dom_proto_named(
            properties,
            "length",
            Property::data(Value::Number(1.0), false, false, true),
        )?;
        self.dom_proto_named(
            properties,
            "name",
            Property::data(Value::String(display), false, false, true),
        )?;
        self.work(8 + NAME.len())?;
        self.charge(std::mem::size_of::<Native>() + 32 + NAME.len())?;
        let mut name = String::new();
        name.try_reserve_exact(NAME.len())
            .map_err(|_| ScriptError::resource("microtask native name allocation failed"))?;
        name.push_str(NAME);
        let properties = std::num::NonZeroUsize::new(properties)
            .ok_or_else(|| ScriptError::resource("microtask metadata ID is zero"))?;
        let function = Value::Native(Rc::new(Native {
            name,
            receiver: Value::Undefined,
            properties: Some(properties),
        }));
        // One pinned B=6 Entry search and insertion. Comparison units follow
        // the existing tree model; these are not processor instructions.
        let (comparisons, height) = tree(self.environments[0].bindings.len());
        let levels = height + 1;
        self.work(8 + comparisons * (1 + NAME.len() / 8) + 28 * levels + NAME.len())?;
        let block = 16 * (std::mem::size_of::<String>() + std::mem::size_of::<Binding>())
            + 32 * std::mem::size_of::<usize>()
            + 64;
        self.charge(32 + NAME.len() + levels * block)?;
        let next_order = self.next_global_order.checked_add(1).ok_or_else(overflow)?;
        let mut key = String::new();
        key.try_reserve_exact(NAME.len())
            .map_err(|_| ScriptError::resource("microtask global key allocation failed"))?;
        key.push_str(NAME);
        let Entry::Vacant(entry) = self.environments[0].bindings.entry(key) else {
            return Err(ScriptError::resource("duplicate microtask global"));
        };
        entry.insert(Binding {
            value: function,
            accessor: None,
            mutable: true,
            initialized: true,
            strict_immutable: false,
            global_property: true,
            global_order: self.next_global_order,
            enumerable: true,
            deletable: true,
        });
        self.next_global_order = next_order;
        Ok(())
    }

    pub fn microtasks_suspended(&self) -> bool {
        self.microtasks.suspended
    }

    // All host-entry epilogues use this even when failure preceded author
    // entry (for example parsing or event construction with an empty FIFO).
    pub(super) fn observe_microtask_failure<T>(&mut self, result: Result<T>) -> Result<T> {
        if result
            .as_ref()
            .is_err_and(|error| error.is_resource_limit() || error.is_unsupported())
        {
            self.microtasks.suspended = true;
        }
        result
    }

    pub(super) fn queue_microtask(&mut self, receiver: &Value, args: &[Value]) -> Result<Value> {
        let result = (|| {
            self.work(8)?;
            // Undefined/null denote the actual realm global, never a lookup
            // of the mutable globalThis property.
            if !matches!(receiver, Value::Undefined | Value::Null | Value::Window) {
                return Err(ScriptError::type_error(
                    "queueMicrotask receiver is not Window",
                ));
            }
            let callback = args
                .first()
                .ok_or_else(|| ScriptError::type_error("queueMicrotask requires a callback"))?;
            if !json_callable(callback) {
                return Err(ScriptError::type_error(
                    "queueMicrotask callback is not callable",
                ));
            }
            if self.microtasks.suspended {
                return Err(ScriptError::resource(
                    "microtask dispatch suspended until reload",
                ));
            }
            let count = self.microtasks.queue.len();
            if count == self.microtasks.admitted_capacity {
                let maximum = MAX_HEAP / std::mem::size_of::<Value>();
                let target = self
                    .microtasks
                    .admitted_capacity
                    .checked_mul(2)
                    .ok_or_else(overflow)?
                    .max(8)
                    .min(maximum);
                if target <= count {
                    return Err(ScriptError::resource(
                        "microtask queue allocation limit exceeded",
                    ));
                }
                let work = count
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(8))
                    .ok_or_else(overflow)?;
                let bytes = target
                    .checked_mul(std::mem::size_of::<Value>())
                    .and_then(|n| n.checked_add(32))
                    .ok_or_else(overflow)?;
                self.work(work)?;
                self.charge(bytes)?;
                self.microtasks
                    .queue
                    .try_reserve_exact(target - count)
                    .map_err(|_| ScriptError::resource("microtask queue allocation failed"))?;
                self.microtasks.admitted_capacity = target;
            }
            self.microtasks.queue.push_back(callback.clone());
            Ok(Value::Undefined)
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn with_author_entry<T>(
        &mut self,
        doc: &mut Document,
        operation: impl FnOnce(&mut Runtime, &mut Document) -> Result<T>,
    ) -> Result<T> {
        let admitted = self.work(8).and_then(|()| {
            self.microtasks
                .author_depth
                .checked_add(1)
                .ok_or_else(overflow)
        });
        let depth = self.observe_microtask_failure(admitted)?;
        self.microtasks.author_depth = depth;
        let result = operation(self, doc);
        self.microtasks.author_depth -= 1;
        let result = self.observe_microtask_failure(result);
        // Ordinary throws still clean up the script. A fatal result has made
        // this a no-op; a checkpoint failure takes precedence over completion.
        self.microtask_checkpoint(doc)?;
        result
    }

    pub(super) fn microtask_checkpoint(&mut self, doc: &mut Document) -> Result<()> {
        if self.microtasks.suspended
            || self.microtasks.author_depth != 0
            || self.microtasks.performing_checkpoint
            || self.microtasks.queue.is_empty()
        {
            return Ok(());
        }
        let admission = self.work(8);
        self.observe_microtask_failure(admission)?;
        self.microtasks.performing_checkpoint = true;
        let result = (|| {
            while !self.microtasks.queue.is_empty() {
                self.work(8)?;
                // Both fixed job debits precede pop. Once popped, even a call
                // preflight failure cannot replay this job in another turn.
                let result = self.with_author_entry(doc, |runtime, doc| {
                    let callback = runtime.microtasks.queue.pop_front().unwrap();
                    runtime.call(callback, Vec::new(), Value::Undefined, doc)
                });
                if let Err(error) = result {
                    if error.is_resource_limit() || error.is_unsupported() {
                        return Err(error);
                    }
                    self.report_microtask_error(error)?;
                }
            }
            Ok(())
        })();
        self.microtasks.performing_checkpoint = false;
        self.observe_microtask_failure(result)
    }

    pub(super) fn report_microtask_error(&mut self, error: ScriptError) -> Result<()> {
        let result = (|| {
            if error.is_resource_limit() || error.is_unsupported() {
                return Err(error);
            }
            let plan = report_plan(
                error.message.len(),
                error.offset,
                self.console.len(),
                self.console.capacity(),
            )?;
            self.work(plan.work)?;
            let Some(rendered) = plan.rendered_bytes else {
                return Ok(());
            };
            self.charge(plan.bytes)?;
            if let Some(target) = plan.target_capacity {
                self.console
                    .try_reserve_exact(target - self.console.len())
                    .map_err(|_| {
                        ScriptError::resource("microtask report slots allocation failed")
                    })?;
            }
            let mut message = String::new();
            message
                .try_reserve_exact(rendered)
                .map_err(|_| ScriptError::resource("microtask report allocation failed"))?;
            message.push_str(REPORT_PREFIX);
            message.push_str(&error.message);
            if let Some(offset) = error.offset {
                write!(message, " at byte {offset}")
                    .map_err(|_| ScriptError::resource("microtask report formatting failed"))?;
            }
            self.console.push(message);
            Ok(())
        })();
        self.observe_microtask_failure(result)
    }
}
