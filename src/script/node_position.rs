//! Ordinary document position for authentic represented nodes. Complete paid
//! parent walks precede alignment; template hosts are not ordinary parents.
//! The body owns no traversal/payload storage and never mutates the document.
use super::*;
use node_predicates::{local_node, node_id};

#[cfg(test)]
mod tests;

pub(super) const METADATA_OBJECTS: usize = 1;

fn invalid() -> ScriptError {
    ScriptError::type_error("invalid Node position tree")
}

// The caller prepays local validation before every use. Also enforce the
// canonical root kind on a self comparison, where no ancestor walk follows.
fn position_local(doc: &Document, id: NodeId) -> Result<&crate::dom::Node> {
    let node = local_node(doc, id)?;
    if id == doc.root && !matches!(node.kind, NodeKind::Document) {
        return Err(invalid());
    }
    Ok(node)
}

// Called only after its eight-work debit in the immutable second phase. Both
// complete walks already checked these links; keep the access panic-free.
fn position_parent(doc: &Document, id: NodeId) -> Result<NodeId> {
    doc.nodes
        .get(id)
        .and_then(|node| node.parent)
        .ok_or_else(invalid)
}

impl Runtime {
    fn position_root_and_depth(
        &mut self,
        mut current: NodeId,
        doc: &Document,
    ) -> Result<(NodeId, usize)> {
        self.work(4)?;
        let mut depth = 0;
        loop {
            self.work(16)?;
            let node = position_local(doc, current)?;
            let Some(parent) = node.parent else {
                self.work(4)?;
                return Ok((current, depth));
            };
            if depth == crate::dom::MAX_DEPTH || parent == current {
                return Err(invalid());
            }
            self.work(12)?;
            let parent_node = local_node(doc, parent)?;
            if !matches!(
                parent_node.kind,
                NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_)
            ) {
                return Err(invalid());
            }
            let scan = parent_node
                .children
                .len()
                .checked_mul(4)
                .and_then(|n| n.checked_add(1))
                .ok_or_else(|| ScriptError::resource("Node position scan overflow"))?;
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
            depth += 1;
            current = parent;
        }
    }

    fn position_branch_order(
        &mut self,
        parent: NodeId,
        receiver_branch: NodeId,
        other_branch: NodeId,
        doc: &Document,
    ) -> Result<Value> {
        self.work(8)?;
        let children = &doc.nodes.get(parent).ok_or_else(invalid)?.children;
        if receiver_branch == other_branch || children.len() > crate::dom::MAX_NODES {
            return Err(invalid());
        }
        let scan = children
            .len()
            .checked_mul(6)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| ScriptError::resource("Node position order overflow"))?;
        self.work(scan)?;
        let mut receiver_index = None;
        let mut other_index = None;
        for (index, &child) in children.iter().enumerate() {
            if child == receiver_branch && receiver_index.replace(index).is_some() {
                return Err(invalid());
            }
            if child == other_branch && other_index.replace(index).is_some() {
                return Err(invalid());
            }
        }
        self.work(4)?;
        match (receiver_index, other_index) {
            (Some(a), Some(b)) => Ok(Value::Number(if b < a { 2.0 } else { 4.0 })),
            _ => Err(invalid()),
        }
    }

    pub(super) fn node_compare_document_position(
        &mut self,
        receiver: Value,
        args: &[Value],
        doc: &Document,
    ) -> Result<Value> {
        self.work(16)?;
        let id = node_id(&receiver, doc)?;
        self.work(4)?;
        let value = args.first().ok_or_else(|| {
            ScriptError::type_error("compareDocumentPosition requires an argument")
        })?;
        self.work(12)?;
        // Required Node is nonnullable; do not use predicate_argument.
        let other = node_id(value, doc)?;
        self.work(16)?;
        position_local(doc, id)?;
        self.work(16)?;
        position_local(doc, other)?;
        self.work(2)?;
        if id == other {
            return Ok(Value::Number(0.0));
        }

        let (root, depth) = self.position_root_and_depth(id, doc)?;
        let (other_root, other_depth) = self.position_root_and_depth(other, doc)?;
        self.work(4)?;
        if root != other_root {
            // Stable within this append-only arena, independent of later moves
            // between distinct roots. Use original operands, never cursors.
            return Ok(Value::Number(if other < id { 35.0 } else { 37.0 }));
        }

        self.work(4)?;
        let (mut a, mut b) = (id, other);
        let (mut da, mut db) = (depth, other_depth);
        loop {
            self.work(2)?;
            if da <= db {
                break;
            }
            self.work(8)?;
            a = position_parent(doc, a)?;
            da -= 1;
        }
        loop {
            self.work(2)?;
            if db <= da {
                break;
            }
            self.work(8)?;
            b = position_parent(doc, b)?;
            db -= 1;
        }
        self.work(4)?;
        if a == b {
            return Ok(Value::Number(if depth < other_depth { 20.0 } else { 10.0 }));
        }
        loop {
            self.work(4)?;
            if da == 0 {
                return Err(invalid());
            }
            self.work(8)?;
            let pa = position_parent(doc, a)?;
            self.work(8)?;
            let pb = position_parent(doc, b)?;
            if pa == pb {
                return self.position_branch_order(pa, a, b, doc);
            }
            a = pa;
            b = pb;
            da -= 1;
        }
    }
}
