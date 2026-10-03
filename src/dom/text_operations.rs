//! Exact Text sibling operations. The caller owns suffix creation and the later
//! source-data replacement; this module never rolls back those earlier phases.
use super::*;
use std::mem::size_of;

#[cfg(test)]
mod tests;

fn add(a: usize, b: usize) -> Result<usize, DomDataError> {
    a.checked_add(b).ok_or(DomDataError::LimitExceeded)
}

fn mul(a: usize, b: usize) -> Result<usize, DomDataError> {
    a.checked_mul(b).ok_or(DomDataError::LimitExceeded)
}

impl Document {
    // Callers prepay the fixed lookup/kind/leaf checks before entering.
    fn text_operation_data(&self, id: NodeId) -> Result<&DomString, DomDataError> {
        match self.nodes.get(id) {
            Some(Node {
                kind: NodeKind::Text(data),
                children,
                ..
            }) if children.is_empty() => Ok(data),
            _ => Err(DomDataError::InvalidNode),
        }
    }

    fn text_operation_children(&self, parent: NodeId) -> Result<&[NodeId], DomDataError> {
        let node = self.nodes.get(parent).ok_or(DomDataError::InvalidNode)?;
        if !matches!(
            node.kind,
            NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_)
        ) || matches!(node.kind, NodeKind::Document) && parent != self.root
        {
            return Err(DomDataError::InvalidNode);
        }
        if node.children.len() > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        Ok(&node.children)
    }

    /// Insert an already created, detached Text beside the source. Each quota
    /// decision and fallible reservation precedes the two-link commit.
    /// Document-parent Text is supported for structurally admitted host trees;
    /// this is internal insertion, not public pre-insertion hierarchy checking.
    pub(crate) fn insert_fresh_text_after(
        &mut self,
        original: NodeId,
        fresh: NodeId,
        budget: &mut DomMutationBudget,
    ) -> Result<(), DomDataError> {
        budget.work(16)?;
        if self.nodes.len() > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        self.text_operation_data(original)?;
        self.text_operation_data(fresh)?;
        if original == fresh || self.nodes[fresh].parent.is_some() {
            return Err(DomDataError::InvalidNode);
        }
        let Some(parent) = self.nodes[original].parent else {
            return Ok(());
        };
        budget.work(8)?;
        let children = self.text_operation_children(parent)?;
        let mut found = None;
        for (index, &child) in children.iter().enumerate() {
            budget.work(12)?;
            let node = self.nodes.get(child).ok_or(DomDataError::InvalidNode)?;
            if child == parent
                || child == fresh
                || child == self.root
                || node.parent != Some(parent)
            {
                return Err(DomDataError::InvalidNode);
            }
            if child == original {
                if found.is_some() {
                    return Err(DomDataError::InvalidNode);
                }
                found = Some(index);
            }
        }
        let position = found.ok_or(DomDataError::InvalidNode)?;

        // The fresh leaf has the original's edge depth, including fragment-host
        // edges. A root at depth zero plus MAX_DEPTH edges is still admitted.
        let mut cursor = Some(parent);
        let mut depth = 0;
        while let Some(id) = cursor {
            budget.work(8)?;
            depth += 1;
            if depth > MAX_DEPTH {
                return Err(DomDataError::LimitExceeded);
            }
            if id == original || id == fresh {
                return Err(DomDataError::InvalidNode);
            }
            let node = self.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
            cursor = match &node.kind {
                NodeKind::Document if id == self.root && node.parent.is_none() => None,
                NodeKind::Element(_) => node.parent,
                NodeKind::DocumentFragment { host } if node.parent.is_none() => {
                    if let Some(host) = *host {
                        budget.work(16)?;
                        if !matches!(self.nodes.get(host).map(|node| &node.kind),
                            Some(NodeKind::Element(element))
                            if element.namespace == Namespace::Html
                                && element.tag == "template"
                                && element.template_contents == Some(id))
                        {
                            return Err(DomDataError::InvalidNode);
                        }
                    }
                    *host
                }
                _ => return Err(DomDataError::InvalidNode),
            };
        }

        let length = self.nodes[parent].children.len();
        let required = add(length, 1)?;
        if required > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        let insertion = add(position, 1)?;
        let shift = length - insertion;
        budget.work(add(8, mul(2, shift)?)?)?;
        if length == self.nodes[parent].children.capacity() {
            budget.work(add(1, mul(2, length)?)?)?;
            budget.charge(mul(required, size_of::<NodeId>())?)?;
            self.nodes[parent]
                .children
                .try_reserve_exact(1)
                .map_err(|_| DomDataError::AllocationFailed)?;
        }
        // No callback, allocation, or Resource return below this point. A fresh
        // Text cannot change base selection, summary IDs, or details groups.
        self.nodes[parent].children.insert(insertion, fresh);
        self.nodes[fresh].parent = Some(parent);
        Ok(())
    }

    /// Concatenate only this Text's contiguous sibling run. Pays for its exact
    /// Vec; Runtime separately admits any subsequent JsString/Rc copy.
    pub(crate) fn whole_text_units_bounded(
        &self,
        id: NodeId,
        maximum_units: usize,
        budget: &mut DomMutationBudget,
    ) -> Result<Vec<u16>, DomDataError> {
        budget.work(16)?;
        if self.nodes.len() > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        self.text_operation_data(id)?;
        let parent = self.nodes[id].parent;
        let single = [id];
        let members = if let Some(parent) = parent {
            budget.work(8)?;
            let children = self.text_operation_children(parent)?;
            let mut position = None;
            for (index, &child) in children.iter().enumerate() {
                budget.work(8)?;
                if child == id {
                    if position.is_some() {
                        return Err(DomDataError::InvalidNode);
                    }
                    position = Some(index);
                }
            }
            let position = position.ok_or(DomDataError::InvalidNode)?;
            let mut start = position;
            let mut end = position + 1;
            while start > 0 {
                budget.work(8)?;
                if !self.text_operation_sibling(children[start - 1], parent)? {
                    break;
                }
                start -= 1;
            }
            while end < children.len() {
                budget.work(8)?;
                if !self.text_operation_sibling(children[end], parent)? {
                    break;
                }
                end += 1;
            }
            &children[start..end]
        } else {
            &single
        };

        let mut units = 0;
        let mut bytes = 0;
        for &member in members {
            budget.work(8)?;
            let data = self.text_operation_data(member)?;
            if self.nodes[member].parent != parent {
                return Err(DomDataError::InvalidNode);
            }
            budget.work(add(1, data.stored_bytes())?)?;
            units = add(units, data.units().count())?;
            bytes = add(bytes, data.stored_bytes())?;
            if units > maximum_units {
                return Err(DomDataError::LimitExceeded);
            }
        }
        // Repeat the selected-node lookups/UTF8 scans and copy each exact unit.
        // The 32-byte header allowance excludes Runtime's separate Rc header.
        budget.work(add(add(8, mul(8, members.len())?)?, add(bytes, units)?)?)?;
        budget.charge(add(32, mul(size_of::<u16>(), units)?)?)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(units)
            .map_err(|_| DomDataError::AllocationFailed)?;
        for &member in members {
            output.extend(self.text_operation_data(member)?.units());
        }
        Ok(output)
    }

    // Paid by each boundary probe. Non-Text siblings stop the run; their own
    // descendants and data are not inspected or included.
    fn text_operation_sibling(&self, id: NodeId, parent: NodeId) -> Result<bool, DomDataError> {
        let node = self.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
        if node.parent != Some(parent) {
            return Err(DomDataError::InvalidNode);
        }
        if matches!(node.kind, NodeKind::Text(_)) {
            if !node.children.is_empty() {
                return Err(DomDataError::InvalidNode);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
