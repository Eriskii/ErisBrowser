//! Ordinary Node.cloneNode over checked, exact represented non-Document copies.
//! The native body prepays its return before atomic Document publication. Later
//! VM result handling may still exhaust; whole evaluation is not rolled back.
use super::*;
use crate::dom::{DomDataError, DomMutationBudget};

#[cfg(test)]
mod tests;

pub(super) const METADATA_OBJECTS: usize = 1;

fn clone_error(error: DomDataError) -> ScriptError {
    match error {
        DomDataError::InvalidNode | DomDataError::InvalidData => {
            ScriptError::type_error("invalid DOM clone source")
        }
        DomDataError::LimitExceeded => ScriptError::resource("DOM clone budget or limit exceeded"),
        DomDataError::AllocationFailed => ScriptError::resource("DOM clone allocation failed"),
    }
}

impl Runtime {
    pub(super) fn install_node_clone(&mut self, prototype: usize) -> Result<()> {
        self.work(4)?;
        let method = self.node_data_function("cloneNode", "cloneNode", 0)?;
        self.dom_proto_named(
            prototype,
            "cloneNode",
            Property::data(method, true, true, true),
        )
    }

    pub(super) fn node_clone_node(
        &mut self,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(16)?;
        let source = node_predicates::node_id(&receiver, doc)?;
        self.work(4)?;
        // Optional boolean conversion has no authored hooks or property reads.
        let deep = args.first().is_some_and(Value::truthy);
        self.work(4)?;
        if matches!(doc.nodes[source].kind, NodeKind::Document) {
            if source != doc.root
                || doc.nodes[source].parent.is_some()
                || doc.nodes.len() > crate::dom::MAX_NODES
            {
                return Err(clone_error(DomDataError::InvalidNode));
            }
            return Err(ScriptError::unsupported(
                "Document cloning is not implemented",
            ));
        }
        // No further native-body admission follows successful publication.
        self.work(2)?;
        let mut budget = DomMutationBudget {
            steps: self.steps,
            allocated: self.allocated,
            heap_limit: MAX_HEAP,
        };
        let result = doc.clone_node_checked(source, deep, &mut budget);
        self.steps = budget.steps;
        self.allocated = budget.allocated;
        result.map(Value::Node).map_err(clone_error)
    }
}
