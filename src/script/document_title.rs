//! Ordinary Document.title accessors. DOM selection and normalized reads are
//! shared with host presentation; stored title children retain the raw input.
use super::*;
use crate::dom::{DomDataError, DomMutationBudget, TitleMode, TitleTarget};

#[cfg(test)]
mod tests;

// Keep separate from the DOM.D*/DOM.E* ParentNode invocation namespace.
pub(super) const PREFIX: &str = "DOM.Title.";
pub(super) const METADATA_OBJECTS: usize = 2;

fn title_error(error: DomDataError) -> ScriptError {
    match error {
        DomDataError::InvalidNode | DomDataError::InvalidData => {
            ScriptError::type_error("invalid Document title tree")
        }
        DomDataError::LimitExceeded => ScriptError::resource("Document title limit exceeded"),
        DomDataError::AllocationFailed => ScriptError::resource("Document title allocation failed"),
    }
}

impl Runtime {
    fn document_title_function(
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
            .map_err(|_| ScriptError::resource("Document title native name allocation failed"))?;
        name.push_str(PREFIX);
        name.push_str(suffix);
        let properties = std::num::NonZeroUsize::new(properties)
            .ok_or_else(|| ScriptError::resource("Document title metadata ID is zero"))?;
        Ok(Value::Native(Rc::new(Native {
            properties: Some(properties),
            name,
            receiver: Value::Undefined,
        })))
    }

    pub(super) fn install_document_title(&mut self, prototype: usize) -> Result<()> {
        self.work(4)?;
        let get = self.document_title_function("get", "get title", 0)?;
        let set = self.document_title_function("set", "set title", 1)?;
        self.dom_proto_named(
            prototype,
            "title",
            Property {
                value: PropertyValue::Accessor { get, set },
                enumerable: true,
                configurable: true,
            },
        )
    }

    pub(super) fn document_title_call_preflight(&mut self, name: &str) -> Result<()> {
        // Invocation copies the owned name only after this debit; failures
        // remain inside the existing machine stack-cleanup boundary.
        self.work(4 + name.len())?;
        self.charge(32 + name.len())
    }

    pub(super) fn document_title_native(
        &mut self,
        method: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8)?;
        let setter = match method {
            "get" => false,
            "set" => true,
            _ => return Err(ScriptError::type_error("unknown Document title accessor")),
        };
        let root = match receiver {
            Value::Document => doc.root,
            Value::Node(id) if id == doc.root => id,
            _ => return Err(ScriptError::type_error("incompatible Document receiver")),
        };
        if !matches!(
            doc.nodes.get(root).map(|node| &node.kind),
            Some(NodeKind::Document)
        ) {
            return Err(ScriptError::type_error("invalid Document receiver"));
        }
        if !setter {
            let mut budget = DomMutationBudget {
                steps: self.steps,
                allocated: self.allocated,
                heap_limit: MAX_HEAP,
            };
            let result = doc.title_units_bounded(MAX_STRING, &mut budget);
            self.steps = budget.steps;
            self.allocated = budget.allocated;
            let units = result.map_err(title_error)?;
            // DOM paid the two scans and Vec; only its final Rc copy remains.
            self.work(1 + units.len())?;
            self.charge(32 + 2 * units.len())?;
            return Ok(Value::String(units.into()));
        }
        let value = args.first().cloned().unwrap_or(Value::Undefined);
        let text = self.string_hint(value, doc)?;
        let mut budget = DomMutationBudget {
            steps: self.steps,
            allocated: self.allocated,
            heap_limit: MAX_HEAP,
        };
        let selected = (|| match doc.title_target(TitleMode::Write, &mut budget)? {
            TitleTarget::None => Ok(None),
            TitleTarget::Existing(id) => Ok(Some(id)),
            TitleTarget::Create {
                parent,
                namespace,
                first,
            } => {
                let id = doc.create_title_element(namespace, &mut budget)?;
                doc.insert_title_element(parent, id, first, &mut budget)?;
                Ok(Some(id))
            }
        })();
        self.steps = budget.steps;
        self.allocated = budget.allocated;
        let Some(id) = selected.map_err(title_error)? else {
            return Ok(Value::Undefined);
        };
        // Creation and insertion are reached before content planning. A later
        // refusal preserves an already inserted empty title and author effects.
        let plan = self.plan_dom_data(text.units().iter().copied(), text.len(), text.len())?;
        self.work(8)?;
        doc.check_text_content_replacement(id, plan.stored_bytes())
            .map_err(title_error)?;
        let data = self.emit_dom_data(plan)?;
        let mut budget = DomMutationBudget {
            steps: self.steps,
            allocated: self.allocated,
            heap_limit: MAX_HEAP,
        };
        let result = doc.replace_text_content_owned(id, data, &mut budget);
        self.steps = budget.steps;
        self.allocated = budget.allocated;
        result.map_err(title_error)?;
        Ok(Value::Undefined)
    }
}
