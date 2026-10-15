//! Streaming call arguments. Admission is logical: allocator spare capacity
//! never grants unpaid argument storage. Iterator errors never perform Close.
use super::{Document, PropertyKey, Result, Runtime, ScriptError, Value, js_object};

#[cfg(test)]
mod allocation_probe;
#[cfg(test)]
mod quota_tests;

const INITIAL_CAPACITY: usize = 8;
const MAX_ARGUMENTS: usize = 65_536;

pub(super) struct Arguments {
    values: Vec<Value>,
    admitted_capacity: usize,
}

// Receipt for an owned value whose move and release were prepaid by the
// reached syntax item or iterator attempt. There is no allocation or Drop hook.
pub(super) struct ReceivedValue(Value);

struct IteratorState {
    iterator: Value,
    next: Value,
}

pub(super) fn prepay_entry(runtime: &mut Runtime) -> Result<()> {
    runtime.work(8)
}

pub(super) fn prepay_item(runtime: &mut Runtime) -> Result<()> {
    // Schedule descriptor read/classification, resume read/classification,
    // receipt and release; all are admitted before evaluating the child.
    runtime.work(6)
}

pub(super) fn receive_prepaid(value: Value) -> ReceivedValue {
    ReceivedValue(value)
}

fn prepay_attempt(runtime: &mut Runtime) -> Result<()> {
    runtime.work(8)
}

fn clone_handle(runtime: &mut Runtime, value: &Value) -> Result<Value> {
    runtime.work(1)?;
    Ok(value.clone())
}

fn fixed_key(runtime: &mut Runtime, name: &str) -> Result<PropertyKey> {
    runtime.dom_proto_text(name).map(Into::into)
}

impl Arguments {
    pub(super) fn empty_prepaid() -> Self {
        Self {
            values: Vec::new(),
            admitted_capacity: 0,
        }
    }

    pub(super) fn append_received(
        &mut self,
        runtime: &mut Runtime,
        ReceivedValue(value): ReceivedValue,
    ) -> Result<()> {
        runtime.work(8)?;
        let length = self.values.len();
        let admitted = self.admitted_capacity;
        if length > admitted
            || admitted > MAX_ARGUMENTS
            || (admitted != 0 && (admitted < INITIAL_CAPACITY || !admitted.is_power_of_two()))
            || self.values.capacity() < admitted
            || length == MAX_ARGUMENTS
        {
            return Err(ScriptError::resource(
                "invalid or exhausted spread argument storage",
            ));
        }
        if length == admitted {
            let target = admitted
                .saturating_mul(2)
                .clamp(INITIAL_CAPACITY, MAX_ARGUMENTS);
            runtime.work(8 + 2 * length)?;
            // Charge the whole new logical buffer cumulatively, even when the
            // allocator already retained enough spare physical capacity.
            runtime.charge(32 + target * std::mem::size_of::<Value>())?;
            #[cfg(test)]
            allocation_probe::before_reserve()?;
            self.values
                .try_reserve_exact(target - length)
                .map_err(|_| ScriptError::resource("spread argument allocation failed"))?;
            #[cfg(test)]
            allocation_probe::completed();
            // No fallible quota operation or callback separates reservation
            // from publication of its logical capacity and incoming value.
            self.admitted_capacity = target;
        }
        self.values.push(value);
        Ok(())
    }

    pub(super) fn spread_received(
        &mut self,
        runtime: &mut Runtime,
        source: ReceivedValue,
        doc: &mut Document,
    ) -> Result<()> {
        let mut iterator = IteratorState::acquire(runtime, source, doc)?;
        while let Some(value) = iterator.next_value(runtime, doc)? {
            self.append_received(runtime, value)?;
        }
        Ok(())
    }

    pub(super) fn into_values(self) -> Vec<Value> {
        self.values
    }
}

impl IteratorState {
    fn acquire(
        runtime: &mut Runtime,
        ReceivedValue(source): ReceivedValue,
        doc: &mut Document,
    ) -> Result<Self> {
        runtime.work(8)?;
        let method = runtime
            .array_from_iterator_method(&source, doc)?
            .ok_or_else(|| ScriptError::type_error("spread argument is not iterable"))?;
        let iterator = runtime.call(method, Vec::new(), source, doc)?;
        if !js_object(&iterator) {
            return Err(ScriptError::type_error("iterator must be an object"));
        }
        let receiver = clone_handle(runtime, &iterator)?;
        let key = fixed_key(runtime, "next")?;
        // Get once. Callability is checked by the first reached Call(next),
        // after acquisition, rather than by an eager separate check.
        let next = runtime.get_property_key(receiver, &key, doc)?;
        Ok(Self { iterator, next })
    }

    fn next_value(
        &mut self,
        runtime: &mut Runtime,
        doc: &mut Document,
    ) -> Result<Option<ReceivedValue>> {
        prepay_attempt(runtime)?;
        runtime.work(2)?;
        let step = runtime.call(self.next.clone(), Vec::new(), self.iterator.clone(), doc)?;
        if !js_object(&step) {
            return Err(ScriptError::type_error("iterator result must be an object"));
        }
        let receiver = clone_handle(runtime, &step)?;
        let done_key = fixed_key(runtime, "done")?;
        if runtime.get_property_key(receiver, &done_key, doc)?.truthy() {
            return Ok(None);
        }
        let value_key = fixed_key(runtime, "value")?;
        runtime
            .get_property_key(step, &value_key, doc)
            .map(|value| Some(receive_prepaid(value)))
    }
}
