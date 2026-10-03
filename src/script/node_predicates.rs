//! Ordinary Node predicates over authentic IDs and current internal tree links.
//! Bodies own no traversal/payload storage. Existing diagnostic and generic VM
//! allocation limitations remain separate; this is not a global graph audit.
use super::*;

#[cfg(test)]
mod tests;

pub(super) const METADATA_OBJECTS: usize = 3;

fn invalid() -> ScriptError {
    ScriptError::type_error("invalid Node predicate tree")
}

// Every caller prepays the fixed brand/conversion or local-shape work.
pub(super) fn node_id(value: &Value, doc: &Document) -> Result<NodeId> {
    match value {
        Value::Document
            if matches!(
                doc.nodes.get(doc.root).map(|node| &node.kind),
                Some(NodeKind::Document)
            ) =>
        {
            Ok(doc.root)
        }
        Value::Node(id) if doc.nodes.get(*id).is_some() => Ok(*id),
        _ => Err(ScriptError::type_error("incompatible Node predicate value")),
    }
}

pub(super) fn local_node(doc: &Document, id: NodeId) -> Result<&crate::dom::Node> {
    let node = doc.nodes.get(id).ok_or_else(invalid)?;
    if doc.nodes.len() > crate::dom::MAX_NODES || node.children.len() > crate::dom::MAX_NODES {
        return Err(invalid());
    }
    match &node.kind {
        NodeKind::Document if id != doc.root || node.parent.is_some() => Err(invalid()),
        NodeKind::DocumentFragment { .. } if node.parent.is_some() => Err(invalid()),
        NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_) => Ok(node),
        _ if node.children.is_empty() => Ok(node),
        _ => Err(invalid()),
    }
}

impl Runtime {
    pub(super) fn install_node_has_child_nodes(&mut self, prototype: usize) -> Result<()> {
        self.work(4)?;
        let method = self.node_data_function("hasChildNodes", "hasChildNodes", 0)?;
        self.dom_proto_named(
            prototype,
            "hasChildNodes",
            Property::data(method, true, true, true),
        )
    }

    pub(super) fn install_node_identity_members(&mut self, prototype: usize) -> Result<()> {
        self.work(4)?;
        for name in ["isSameNode", "contains"] {
            let method = self.node_data_function(name, name, 1)?;
            self.dom_proto_named(prototype, name, Property::data(method, true, true, true))?;
        }
        Ok(())
    }

    fn predicate_argument(&mut self, args: &[Value], doc: &Document) -> Result<Option<NodeId>> {
        self.work(4)?;
        let value = args
            .first()
            .ok_or_else(|| ScriptError::type_error("Node predicate requires an argument"))?;
        self.work(12)?;
        match value {
            Value::Undefined | Value::Null => Ok(None),
            _ => node_id(value, doc).map(Some),
        }
    }

    pub(super) fn node_has_child_nodes(
        &mut self,
        receiver: Value,
        doc: &Document,
    ) -> Result<Value> {
        self.work(16)?;
        let id = node_id(&receiver, doc)?;
        self.work(8)?;
        let node = local_node(doc, id)?;
        self.work(1)?;
        Ok(Value::Bool(!node.children.is_empty()))
    }

    pub(super) fn node_is_same_node(
        &mut self,
        receiver: Value,
        args: &[Value],
        doc: &Document,
    ) -> Result<Value> {
        self.work(16)?;
        let id = node_id(&receiver, doc)?;
        let other = self.predicate_argument(args, doc)?;
        self.work(1)?;
        Ok(Value::Bool(other == Some(id)))
    }

    pub(super) fn node_contains(
        &mut self,
        receiver: Value,
        args: &[Value],
        doc: &Document,
    ) -> Result<Value> {
        self.work(16)?;
        let id = node_id(&receiver, doc)?;
        let Some(mut current) = self.predicate_argument(args, doc)? else {
            self.work(1)?;
            return Ok(Value::Bool(false));
        };
        let mut edges = 0;
        loop {
            self.work(12)?;
            let node = local_node(doc, current)?;
            if current == id {
                return Ok(Value::Bool(true));
            }
            let Some(parent) = node.parent else {
                return Ok(Value::Bool(false));
            };
            if edges == crate::dom::MAX_DEPTH || parent == current {
                return Err(invalid());
            }
            self.work(12)?;
            let parent_node = local_node(doc, parent)?;
            if !matches!(
                parent_node.kind,
                NodeKind::Document | NodeKind::Element(_) | NodeKind::DocumentFragment { .. }
            ) {
                return Err(invalid());
            }
            let scan = parent_node
                .children
                .len()
                .checked_mul(4)
                .and_then(|n| n.checked_add(1))
                .ok_or_else(|| ScriptError::resource("Node predicate scan overflow"))?;
            self.work(scan)?;
            let mut found = false;
            for &child in &parent_node.children {
                if child == current {
                    if found {
                        return Err(invalid());
                    }
                    found = true;
                }
            }
            if !found {
                return Err(invalid());
            }
            // The bound is checked before advancing; the next iteration still
            // processes an ancestor at exactly MAX_DEPTH ordinary edges.
            edges += 1;
            current = parent;
        }
    }
}
