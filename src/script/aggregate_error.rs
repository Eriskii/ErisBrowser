//! AggregateError construction with ordered author observations and prepaid
//! temporary ownership. Iterator failures propagate without IteratorClose.
use super::*;
use std::mem::size_of;

mod install;

pub(super) const METADATA_OBJECTS: usize = 2;

struct AggregateIterator {
    iterator: Value,
    next: Value,
}

// Only a reached, prepaid iterator attempt creates this ownership receipt.
struct ReceivedAggregateValue(Value);

struct AggregateList {
    values: Vec<Value>,
    admitted_capacity: usize,
}

impl Runtime {
    pub(super) fn aggregate_error_call(
        &mut self,
        native: &Rc<Native>,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let result = (|| {
            // The incoming target must be releasable if body admission fails.
            self.work(2)?;
            self.aggregate_error_body(arguments, Value::Native(native.clone()), doc)
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn aggregate_error_construct(
        &mut self,
        arguments: &[Value],
        new_target: &Value,
        doc: &mut Document,
    ) -> Result<Value> {
        let result = (|| {
            self.work(2)?;
            self.aggregate_error_body(arguments, new_target.clone(), doc)
        })();
        self.observe_microtask_failure(result)
    }

    fn aggregate_error_body(
        &mut self,
        arguments: &[Value],
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        // Bounded control and outer temporary cleanup precede author callbacks.
        self.work(16)?;
        let mut prototype = self.aggregate_error_named_get(new_target, "prototype", doc)?;
        if !js_object(&prototype) {
            let (comparisons, _) = promises::intrinsic_tree(self.prototypes.len());
            self.work(comparisons * (1 + "AggregateError".len() / 8))?;
            let id = self
                .prototypes
                .get("AggregateError")
                .copied()
                .filter(|id| *id < self.objects.len())
                .ok_or_else(|| ScriptError::resource("AggregateError prototype missing"))?;
            prototype = Value::Object(id);
        }
        let object = self.new_aggregate_error_object(prototype)?;
        let message = arguments.get(1).unwrap_or(&Value::Undefined);
        if !matches!(message, Value::Undefined) {
            self.work(1)?;
            let text = self.string_hint(message.clone(), doc)?;
            self.define_aggregate_error_data(object, "message", Value::String(text))?;
        }
        let options = arguments.get(2).unwrap_or(&Value::Undefined);
        self.install_aggregate_error_cause(object, options, doc)?;
        let items = arguments.first().unwrap_or(&Value::Undefined);
        let values = self.aggregate_error_collect(items, doc)?;
        // The existing helper keeps its complete accounting and terminal
        // observer. Moving this Vec gives no refund for the temporary list.
        let array = self.new_all_values_array(values)?;
        self.define_aggregate_error_data(object, "errors", array)?;
        Ok(Value::Object(object))
    }

    pub(super) fn intrinsic_aggregate_error_from_list(
        &mut self,
        values: Vec<Value>,
    ) -> Result<Value> {
        let result = (|| {
            // Four bounded transitions and four local ownership roles. The
            // producer prepaid the moved list's per-element lifetime costs.
            self.work(8)?;
            let (comparisons, _) = promises::intrinsic_tree(self.prototypes.len());
            self.work(comparisons * (1 + "AggregateError".len() / 8))?;
            let prototype = self
                .prototypes
                .get("AggregateError")
                .copied()
                .filter(|id| *id < self.objects.len())
                .ok_or_else(|| ScriptError::resource("AggregateError prototype missing"))?;
            let object = self.new_aggregate_error_object(Value::Object(prototype))?;
            let array = self.new_all_values_array(values)?;
            self.define_aggregate_error_data(object, "errors", array)?;
            Ok(Value::Object(object))
        })();
        self.observe_microtask_failure(result)
    }

    fn aggregate_error_named_get(
        &mut self,
        receiver: Value,
        name: &str,
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(2)?;
        let key = self.dom_proto_text(name)?.into();
        self.get_property_key(receiver, &key, doc)
    }

    fn new_aggregate_error_object(&mut self, prototype: Value) -> Result<usize> {
        self.work(8)?;
        if !js_object(&prototype) {
            return Err(ScriptError::resource("invalid AggregateError prototype"));
        }
        let plan = promises::object_slot_plan(self.objects.len(), self.objects.capacity(), 1)?;
        self.work(plan.work)?;
        self.charge(plan.bytes)?;
        if let Some(target) = plan.target {
            self.objects
                .try_reserve_exact(target - self.objects.len())
                .map_err(|_| ScriptError::resource("AggregateError object allocation failed"))?;
        }
        let id = self.dom_proto_object(Some(prototype), 3)?;
        self.objects[id].intrinsic_error = Some("AggregateError");
        Ok(id)
    }

    fn define_aggregate_error_data(
        &mut self,
        object: usize,
        name: &str,
        value: Value,
    ) -> Result<()> {
        self.work(4)?;
        let owner = self
            .objects
            .get(object)
            .ok_or_else(|| ScriptError::resource("invalid AggregateError object"))?;
        if owner.intrinsic_error != Some("AggregateError") {
            return Err(ScriptError::resource("invalid AggregateError object brand"));
        }
        let key = self.dom_proto_text(name)?.into();
        self.dom_proto_property(object, key, Property::data(value, true, false, true))
    }

    fn install_aggregate_error_cause(
        &mut self,
        object: usize,
        options: &Value,
        doc: &mut Document,
    ) -> Result<()> {
        self.work(10)?;
        if !js_object(options) {
            return Ok(());
        }
        let key = self.dom_proto_text("cause")?.into();
        // HasProperty does not call a getter. Presence, including an inherited
        // undefined value or a setter-only accessor, precedes the separate Get.
        if self.find_property_key_in(options, &key, doc)?.is_some() {
            self.work(1)?;
            let value = self.get_property_key(options.clone(), &key, doc)?;
            // Own publication has a separately paid key lifetime.
            self.define_aggregate_error_data(object, "cause", value)?;
        }
        Ok(())
    }

    fn aggregate_error_collect(
        &mut self,
        source: &Value,
        doc: &mut Document,
    ) -> Result<Vec<Value>> {
        self.work(8)?;
        let mut list = AggregateList::empty_prepaid();
        let iterator = AggregateIterator::acquire(self, source, doc)?;
        while let Some(value) = iterator.next_value(self, doc)? {
            list.append_received(self, value)?;
        }
        Ok(list.into_values())
    }
}

impl AggregateIterator {
    fn acquire(runtime: &mut Runtime, source: &Value, doc: &mut Document) -> Result<Self> {
        runtime.work(8)?;
        let method = runtime
            .array_from_iterator_method(source, doc)?
            .ok_or_else(|| ScriptError::type_error("AggregateError errors are not iterable"))?;
        runtime.work(1)?;
        let iterator = runtime.call(method, Vec::new(), source.clone(), doc)?;
        if !js_object(&iterator) {
            return Err(ScriptError::type_error("iterator is not an object"));
        }
        runtime.work(1)?;
        let next = runtime.aggregate_error_named_get(iterator.clone(), "next", doc)?;
        Ok(Self { iterator, next })
    }

    fn next_value(
        &self,
        runtime: &mut Runtime,
        doc: &mut Document,
    ) -> Result<Option<ReceivedAggregateValue>> {
        // This attempt owns cleanup even if a later append debit refuses.
        runtime.work(8)?;
        runtime.work(2)?;
        let step = runtime.call(self.next.clone(), Vec::new(), self.iterator.clone(), doc)?;
        if !js_object(&step) {
            return Err(ScriptError::type_error("iterator result is not an object"));
        }
        runtime.work(1)?;
        let done = runtime.aggregate_error_named_get(step.clone(), "done", doc)?;
        if done.truthy() {
            return Ok(None);
        }
        let value = runtime.aggregate_error_named_get(step, "value", doc)?;
        Ok(Some(ReceivedAggregateValue(value)))
    }
}

impl AggregateList {
    fn empty_prepaid() -> Self {
        Self {
            values: Vec::new(),
            admitted_capacity: 0,
        }
    }

    fn append_received(
        &mut self,
        runtime: &mut Runtime,
        ReceivedAggregateValue(value): ReceivedAggregateValue,
    ) -> Result<()> {
        runtime.work(8)?;
        let len = self.values.len();
        let admitted = self.admitted_capacity;
        let maximum = MAX_HEAP / size_of::<Value>();
        if len > admitted
            || admitted > maximum
            || self.values.capacity() < admitted
            || (admitted != 0
                && admitted != maximum
                && (admitted < 8 || !admitted.is_power_of_two()))
        {
            return Err(ScriptError::resource("invalid AggregateError list storage"));
        }
        let plan = promises::growth_plan::<Value>(len, admitted, 1)?;
        runtime.work(plan.work)?;
        runtime.charge(plan.bytes)?;
        if let Some(target) = plan.target {
            self.values
                .try_reserve_exact(target - len)
                .map_err(|_| ScriptError::resource("AggregateError list allocation failed"))?;
            self.admitted_capacity = target;
        }
        // All validation, debit and reserve stages precede publication.
        self.values.push(value);
        Ok(())
    }

    fn into_values(self) -> Vec<Value> {
        self.values
    }
}

#[cfg(test)]
mod quota_tests;
