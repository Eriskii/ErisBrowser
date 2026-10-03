//! Checked removal of a callback-free, already unique normalize Text range.
//!
//! Runtime owns the descendant walk and reached-ID uniqueness proof. It may
//! already have replaced the survivor's payload before this separate stage.
//! Refusal never rolls back that earlier replacement or any completed run.
use super::*;

#[cfg(test)]
mod tests;

fn add(left: usize, right: usize) -> Result<usize, DomDataError> {
    left.checked_add(right).ok_or(DomDataError::LimitExceeded)
}

fn mul(left: usize, right: usize) -> Result<usize, DomDataError> {
    left.checked_mul(right).ok_or(DomDataError::LimitExceeded)
}

impl Document {
    /// Detach a nonempty half-open range of Text leaves, compacting once.
    ///
    /// The caller must prove that the reached child IDs are unique before this
    /// immediate callback-free stage. That proof is not repeated here; this is
    /// not a general graph validator. With a survivor, it must be the preceding
    /// Text child. Runtime uses no survivor for its empty-Text removal action.
    /// This method checks the fresh local bounds, kinds and backlinks itself.
    /// It never reads through authored properties or changes any payload.
    pub(crate) fn remove_normalize_text_range(
        &mut self,
        parent: NodeId,
        start: usize,
        end: usize,
        survivor: Option<NodeId>,
        budget: &mut DomMutationBudget,
    ) -> Result<(), DomDataError> {
        budget.work(16)?;
        if self.nodes.len() > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        let node = self.nodes.get(parent).ok_or(DomDataError::InvalidNode)?;
        let valid_parent = match node.kind {
            NodeKind::Document => parent == self.root && node.parent.is_none(),
            NodeKind::DocumentFragment { .. } => node.parent.is_none(),
            NodeKind::Element(_) => true,
            _ => false,
        };
        if !valid_parent || start >= end || end > node.children.len() {
            return Err(DomDataError::InvalidNode);
        }
        let length = node.children.len();
        if length > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        if let Some(id) = survivor {
            budget.work(12)?;
            if start == 0 || self.nodes[parent].children[start - 1] != id {
                return Err(DomDataError::InvalidNode);
            }
            let node = self.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
            if id == parent
                || id == self.root
                || node.parent != Some(parent)
                || !node.children.is_empty()
                || !matches!(node.kind, NodeKind::Text(_))
            {
                return Err(DomDataError::InvalidNode);
            }
        }
        for index in start..end {
            budget.work(12)?;
            let id = self.nodes[parent].children[index];
            let node = self.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
            if id == parent
                || id == self.root
                || Some(id) == survivor
                || node.parent != Some(parent)
                || !node.children.is_empty()
                || !matches!(node.kind, NodeKind::Text(_))
            {
                return Err(DomDataError::InvalidNode);
            }
        }
        let removed = end - start;
        let shifted = length - end;
        let new_length = length - removed;
        budget.work(add(8, add(mul(3, removed)?, mul(2, shifted)?)?)?)?;

        // Every fallible operation has finished. Link writes follow tree order;
        // each surviving suffix ID is moved once. Detached payloads/IDs remain
        // in the arena, and child-vector capacity and retained bytes stay put.
        // Text leaves cannot change selected base/summary element identities,
        // named details groups, or toggle state, so no metadata work is needed.
        for index in start..end {
            let id = self.nodes[parent].children[index];
            self.nodes[id].parent = None;
        }
        self.nodes[parent].children.copy_within(end..length, start);
        self.nodes[parent].children.truncate(new_length);
        Ok(())
    }
}
