//! Ordinary Node data accessors over authentic represented DOM nodes.
//! Mutation observers, ranges, custom-element reactions and unrepresented node
//! kinds remain separate binding gaps; mutable prototypes never confer a brand.
use super::*;
use crate::dom::DomMutationBudget;
use processing_instruction::dom_data_error;

#[cfg(test)]
mod tests;

pub(super) const PREFIX: &str = "DOM.Node.";
pub(super) const METADATA_OBJECTS: usize = 4;

#[derive(Clone, Copy)]
enum DataKind {
    Ignored,
    Character,
    Container,
}
fn data_kind(id: NodeId, doc: &Document) -> Result<DataKind> {
    match doc.nodes.get(id).map(|node| &node.kind) {
        Some(NodeKind::Document | NodeKind::Doctype(_)) => Ok(DataKind::Ignored),
        Some(NodeKind::Text(_) | NodeKind::Comment(_) | NodeKind::ProcessingInstruction { .. }) => {
            Ok(DataKind::Character)
        }
        Some(NodeKind::Element(_) | NodeKind::DocumentFragment { .. }) => Ok(DataKind::Container),
        None => Err(ScriptError::type_error("invalid Node receiver")),
    }
}

impl Runtime {
    pub(super) fn node_data_function(
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
            .map_err(|_| ScriptError::resource("Node accessor name allocation failed"))?;
        name.push_str(PREFIX);
        name.push_str(suffix);
        let properties = std::num::NonZeroUsize::new(properties)
            .ok_or_else(|| ScriptError::resource("Node accessor metadata ID is zero"))?;
        Ok(Value::Native(Rc::new(Native {
            properties: Some(properties),
            name,
            receiver: Value::Undefined,
        })))
    }

    pub(super) fn install_node_data_members(&mut self, prototype: usize) -> Result<()> {
        self.work(4)?;
        for (property, get, set, getter, setter) in [
            (
                "nodeValue",
                "getNodeValue",
                "setNodeValue",
                "get nodeValue",
                "set nodeValue",
            ),
            (
                "textContent",
                "getTextContent",
                "setTextContent",
                "get textContent",
                "set textContent",
            ),
        ] {
            let get = self.node_data_function(get, getter, 0)?;
            let set = self.node_data_function(set, setter, 1)?;
            self.dom_proto_named(
                prototype,
                property,
                Property {
                    value: PropertyValue::Accessor { get, set },
                    enumerable: true,
                    configurable: true,
                },
            )?;
        }
        Ok(())
    }

    pub(super) fn node_data_call_preflight(&mut self, name: &str) -> Result<()> {
        // The invocation bridge copies this name only after admission and keeps
        // any failure within its stack-cleanup boundary.
        self.work(4 + name.len())?;
        self.charge(32 + name.len())
    }

    pub(super) fn node_data_native(
        &mut self,
        method: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        // One reached length check for existing accessors; only the new
        // nine-byte operation name reaches its separately paid comparison.
        self.work(1)?;
        if method.len() == 9 {
            self.work(9)?;
            if method == "normalize" {
                return self.node_normalize(receiver, doc);
            }
        }
        self.work(8)?;
        let (setter, content) = match method {
            "getNodeValue" => (false, false),
            "setNodeValue" => (true, false),
            "getTextContent" => (false, true),
            "setTextContent" => (true, true),
            _ => return Err(ScriptError::type_error("unknown Node accessor")),
        };
        let id = match receiver {
            Value::Document
                if matches!(
                    doc.nodes.get(doc.root).map(|n| &n.kind),
                    Some(NodeKind::Document)
                ) =>
            {
                doc.root
            }
            Value::Node(id) => id,
            _ => return Err(ScriptError::type_error("incompatible Node receiver")),
        };
        let kind = data_kind(id, doc)?;
        if !setter {
            return match kind {
                DataKind::Character => Ok(Value::String(self.dom_text_units(id, doc)?.into())),
                DataKind::Container if content => {
                    Ok(Value::String(self.dom_text_units(id, doc)?.into()))
                }
                _ => Ok(Value::Null),
            };
        }
        // Nullable DOMString converts raw missing/undefined/null to null, which
        // both Node setters treat as empty. An object yielding null is instead
        // converted to the ordinary string "null". Even ignored kinds convert.
        let text = match args.first() {
            None | Some(Value::Undefined | Value::Null) => JsString::default(),
            Some(value) => self.string_hint(value.clone(), doc)?,
        };
        self.work(8)?;
        let kind = data_kind(id, doc)?;
        if matches!(kind, DataKind::Ignored) || matches!(kind, DataKind::Container) && !content {
            return Ok(Value::Undefined);
        }
        let plan = self.plan_dom_data(text.units().iter().copied(), text.len(), text.len())?;
        self.work(8)?;
        match kind {
            DataKind::Character => {
                doc.check_character_data_replacement(id, plan.stored_bytes())
                    .map_err(dom_data_error)?;
                let data = self.emit_dom_data(plan)?;
                self.work(8)?;
                doc.replace_character_data(id, data)
                    .map_err(dom_data_error)?;
            }
            DataKind::Container => {
                doc.check_text_content_replacement(id, plan.stored_bytes())
                    .map_err(dom_data_error)?;
                let data = self.emit_dom_data(plan)?;
                let mut budget = DomMutationBudget {
                    steps: self.steps,
                    allocated: self.allocated,
                    heap_limit: MAX_HEAP,
                };
                let result = doc.replace_text_content_owned(id, data, &mut budget);
                // Copy both counters before propagating either outcome. Failed
                // preparation retains its work and cumulative storage debits.
                self.steps = budget.steps;
                self.allocated = budget.allocated;
                result.map_err(dom_data_error)?;
            }
            DataKind::Ignored => unreachable!(),
        }
        Ok(Value::Undefined)
    }
}
