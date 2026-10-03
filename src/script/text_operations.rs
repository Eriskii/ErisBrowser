//! Exact represented Text operations. CDATA, live ranges and mutation records
//! remain separate gaps. Ordinary prototype changes never confer a Text brand.
use super::*;
use crate::dom::{DomMutationBudget, DomString};
use processing_instruction::dom_data_error;

#[cfg(test)]
mod tests;

pub(super) const PREFIX: &str = "DOM.Text.";
pub(super) const METADATA_OBJECTS: usize = 2;

fn current_text(doc: &Document, id: NodeId) -> Result<&DomString> {
    match doc.nodes.get(id).map(|node| &node.kind) {
        Some(NodeKind::Text(data)) => Ok(data),
        _ => Err(ScriptError::type_error("incompatible Text receiver")),
    }
}
fn overflow() -> ScriptError {
    ScriptError::resource("Text operation work or storage overflow")
}

impl Runtime {
    fn text_operations_function(
        &mut self,
        suffix: &str,
        display: &str,
        length: usize,
    ) -> Result<Value> {
        let properties =
            self.dom_proto_object(Some(Value::Function(self.function_prototype)), 2)?;
        let name = self.dom_proto_text(display)?;
        self.dom_proto_named(
            properties,
            "length",
            Property::data(Value::Number(length as f64), false, false, true),
        )?;
        self.dom_proto_named(
            properties,
            "name",
            Property::data(Value::String(name), false, false, true),
        )?;
        let length = PREFIX.len() + suffix.len();
        self.work(8 + length)?;
        self.charge(std::mem::size_of::<Native>() + 32 + length)?;
        let mut name = String::new();
        name.try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("Text native name allocation failed"))?;
        name.push_str(PREFIX);
        name.push_str(suffix);
        let properties = std::num::NonZeroUsize::new(properties)
            .ok_or_else(|| ScriptError::resource("Text metadata ID is zero"))?;
        Ok(Value::Native(Rc::new(Native {
            properties: Some(properties),
            name,
            receiver: Value::Undefined,
        })))
    }

    pub(super) fn install_text_operations(&mut self, prototype: usize) -> Result<()> {
        self.work(4)?;
        // Web IDL installs attributes before operations, then constructor.
        let get = self.text_operations_function("getWholeText", "get wholeText", 0)?;
        self.dom_proto_named(
            prototype,
            "wholeText",
            Property {
                value: PropertyValue::Accessor {
                    get,
                    set: Value::Undefined,
                },
                enumerable: true,
                configurable: true,
            },
        )?;
        let split = self.text_operations_function("splitText", "splitText", 1)?;
        self.dom_proto_named(
            prototype,
            "splitText",
            Property::data(split, true, true, true),
        )
    }

    pub(super) fn text_operations_call_preflight(&mut self, name: &str) -> Result<()> {
        // Keep the invocation bridge's temporary owned name within its
        // stack-cleanup boundary, including refusal before the copy.
        self.work(4 + name.len())?;
        self.charge(32 + name.len())
    }

    pub(super) fn text_operations_native(
        &mut self,
        method: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(
            method
                .len()
                .checked_mul(2)
                .and_then(|work| work.checked_add(8))
                .ok_or_else(overflow)?,
        )?;
        let split = match method {
            "getWholeText" => false,
            "splitText" => true,
            _ => return Err(ScriptError::type_error("unknown Text operation")),
        };
        let Value::Node(id) = receiver else {
            return Err(ScriptError::type_error("incompatible Text receiver"));
        };
        current_text(doc, id)?;
        if !split {
            return self.whole_text(id, doc);
        }
        let Some(offset) = args.first() else {
            return Err(ScriptError::type_error("missing splitText offset"));
        };
        // Web IDL unsigned long: required argument, Number hint, wrapping
        // conversion. All callbacks finish before data/length/parent reads.
        let offset = to_i32(self.number_value(offset.clone(), doc)?) as u32 as usize;
        self.split_text(id, offset, doc)
    }

    fn text_index_error(&mut self) -> Result<ScriptError> {
        let name = self.dom_proto_text("IndexSizeError")?;
        let message = self.dom_proto_text("offset exceeds Text length")?;
        let value = self.dom_exception(name, message)?;
        self.thrown_error(value)
    }

    fn split_text(&mut self, id: NodeId, offset: usize, doc: &mut Document) -> Result<Value> {
        let old = current_text(doc, id)?;
        self.work(1 + old.stored_bytes())?;
        let length = old.units().count();
        if offset > length {
            return Err(self.text_index_error()?);
        }
        let fresh = self.split_text_suffix(id, offset, length, doc)?;
        let mut budget = DomMutationBudget {
            steps: self.steps,
            allocated: self.allocated,
            heap_limit: MAX_HEAP,
        };
        let inserted = doc.insert_fresh_text_after(id, fresh, &mut budget);
        self.steps = budget.steps;
        self.allocated = budget.allocated;
        inserted.map_err(dom_data_error)?;
        // Insertion has no author callback and does not mutate source data.
        // Reacquire its borrow after the new node/child-vector publications.
        self.split_text_prefix(id, offset, doc)?;
        Ok(Value::Node(fresh))
    }

    fn split_text_suffix(
        &mut self,
        id: NodeId,
        offset: usize,
        length: usize,
        doc: &mut Document,
    ) -> Result<NodeId> {
        let old = current_text(doc, id)?;
        let count = length.checked_sub(offset).ok_or_else(overflow)?;
        // Skip consumes the original prefix too. No public substring/JS
        // string temporary: DOM node data has its own storage admission.
        let plan = self.plan_dom_data(old.units().skip(offset), count, old.stored_bytes())?;
        self.work(8)?;
        if !doc.admits_text_node(plan.stored_bytes()) {
            return Err(ScriptError::resource(
                "Text suffix node or storage limit exceeded",
            ));
        }
        self.ensure_dom_capacity(doc, 1)?;
        let data = self.emit_dom_data(plan)?;
        self.work(8)?;
        // Actual arena reserve is last: no later quota check can leave changed
        // arena capacity without the newly published suffix Text.
        self.dom_reserve_node_growth(doc, 1)?;
        doc.create_text_node_owned(data).map_err(dom_data_error)
    }

    fn split_text_prefix(&mut self, id: NodeId, offset: usize, doc: &mut Document) -> Result<()> {
        let old = current_text(doc, id)?;
        let source = old
            .stored_bytes()
            .min(offset.checked_mul(4).ok_or_else(overflow)?);
        let plan = self.plan_dom_data(old.units().take(offset), offset, source)?;
        self.work(8)?;
        doc.check_character_data_replacement(id, plan.stored_bytes())
            .map_err(dom_data_error)?;
        let data = self.emit_dom_data(plan)?;
        self.work(8)?;
        doc.replace_character_data(id, data).map_err(dom_data_error)
    }

    fn whole_text(&mut self, id: NodeId, doc: &Document) -> Result<Value> {
        let mut budget = DomMutationBudget {
            steps: self.steps,
            allocated: self.allocated,
            heap_limit: MAX_HEAP,
        };
        let units = doc.whole_text_units_bounded(id, MAX_STRING, &mut budget);
        // DOM owns the Vec allocation and both source passes. Copy counters
        // on error as well as success; Runtime separately pays the Rc copy.
        self.steps = budget.steps;
        self.allocated = budget.allocated;
        let units = units.map_err(dom_data_error)?;
        self.work(units.len().checked_add(1).ok_or_else(overflow)?)?;
        self.charge(
            units
                .len()
                .checked_mul(2)
                .and_then(|bytes| bytes.checked_add(32))
                .ok_or_else(overflow)?,
        )?;
        Ok(Value::String(units.into()))
    }
}
