//! Paid normalization of ordinary descendants. Removed Text nodes retain their
//! exact data/identity; ranges, mutation records and custom reactions are gaps.
use super::*;
use crate::dom::{DomMutationBudget, DomString, DomUnits};
use processing_instruction::dom_data_error;
use std::collections::btree_map::Entry;
use std::mem::size_of;

#[cfg(test)]
mod tests;

pub(super) const METADATA_OBJECTS: usize = 1;

fn overflow() -> ScriptError {
    ScriptError::resource("normalize work or storage overflow")
}
fn invalid() -> ScriptError {
    ScriptError::type_error("invalid normalize tree")
}
fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or_else(overflow)
}
fn mul(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or_else(overflow)
}

#[derive(Clone, Copy)]
struct NormalizeFrame {
    parent: NodeId,
    next: usize,
}

// All lookups in these two helpers have a fixed debit at their caller.
fn shape(doc: &Document, id: NodeId) -> Result<bool> {
    let node = doc.nodes.get(id).ok_or_else(invalid)?;
    if node.children.len() > crate::dom::MAX_NODES {
        return Err(overflow());
    }
    match &node.kind {
        NodeKind::Document if id != doc.root || node.parent.is_some() => Err(invalid()),
        NodeKind::DocumentFragment { .. } if node.parent.is_some() => Err(invalid()),
        NodeKind::Document | NodeKind::Element(_) | NodeKind::DocumentFragment { .. } => Ok(true),
        _ if node.children.is_empty() => Ok(false),
        _ => Err(invalid()),
    }
}
fn text_data(doc: &Document, id: NodeId) -> Result<&DomString> {
    match doc.nodes.get(id) {
        Some(crate::dom::Node {
            kind: NodeKind::Text(data),
            children,
            ..
        }) if children.is_empty() => Ok(data),
        _ => Err(invalid()),
    }
}

struct NormalizeRun<'d> {
    doc: &'d Document,
    members: &'d [NodeId],
    units: usize,
    source_bytes: usize,
}

#[derive(Clone)]
struct NormalizeUnits<'d> {
    doc: &'d Document,
    members: std::slice::Iter<'d, NodeId>,
    current: Option<DomUnits<'d>>,
}

impl<'d> NormalizeRun<'d> {
    fn units(&self) -> NormalizeUnits<'d> {
        NormalizeUnits {
            doc: self.doc,
            members: self.members.iter(),
            current: None,
        }
    }
}

impl Iterator for NormalizeUnits<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        loop {
            if let Some(current) = &mut self.current
                && let Some(unit) = current.next()
            {
                return Some(unit);
            }
            let &id = self.members.next()?;
            // Only normalize_run constructs this private stream, after all
            // IDs/kinds are checked. The shared Document borrow prevents any
            // mutation throughout both builder passes; no callback runs here.
            let NodeKind::Text(data) = &self.doc.nodes[id].kind else {
                unreachable!("validated normalize run changed kind")
            };
            self.current = Some(data.units());
        }
    }
}

impl Runtime {
    pub(super) fn install_node_normalize(&mut self, prototype: usize) -> Result<()> {
        self.work(4)?;
        let method = self.node_data_function("normalize", "normalize", 0)?;
        self.dom_proto_named(
            prototype,
            "normalize",
            Property::data(method, true, true, true),
        )
    }

    // The existing DOM.Node invocation preflight owns the temporary native
    // name copy, inside machine/calls' stack-cleanup boundary.
    pub(super) fn node_normalize(&mut self, receiver: Value, doc: &mut Document) -> Result<Value> {
        self.work(16)?;
        let id = match receiver {
            Value::Document
                if matches!(
                    doc.nodes.get(doc.root).map(|node| &node.kind),
                    Some(NodeKind::Document)
                ) =>
            {
                doc.root
            }
            Value::Node(id) if doc.nodes.get(id).is_some() => id,
            _ => return Err(ScriptError::type_error("incompatible Node receiver")),
        };
        self.work(8)?;
        if doc.nodes.len() > crate::dom::MAX_NODES {
            return Err(overflow());
        }
        if !shape(doc, id)? || doc.nodes[id].children.is_empty() {
            return Ok(Value::Undefined);
        }

        self.work(8)?;
        self.charge(32)?;
        let mut frames = Vec::new();
        let mut reached = BTreeMap::new();
        self.normalize_mark(&mut reached, id)?;
        self.normalize_push(
            &mut frames,
            NormalizeFrame {
                parent: id,
                next: 0,
            },
        )?;

        while !frames.is_empty() {
            self.work(8)?;
            let frame_index = frames.len() - 1;
            let NormalizeFrame { parent, next } = frames[frame_index];
            if next == doc.nodes[parent].children.len() {
                self.work(1)?;
                frames.pop();
                continue;
            }
            self.work(12)?;
            let child = *doc.nodes[parent].children.get(next).ok_or_else(invalid)?;
            let node = doc.nodes.get(child).ok_or_else(invalid)?;
            if node.parent != Some(parent) || child == doc.root {
                return Err(invalid());
            }
            // The root frame is depth zero. A child at exactly MAX_DEPTH is
            // admitted; this operation cannot grow the tree's depth.
            if frames.len() > crate::dom::MAX_DEPTH {
                return Err(overflow());
            }
            let next_index = add(next, 1)?;
            let container = shape(doc, child)?;
            self.normalize_mark(&mut reached, child)?;
            if matches!(node.kind, NodeKind::Text(_)) {
                if text_data(doc, child)?.stored_bytes() == 0 {
                    self.normalize_remove(doc, parent, next, next_index, None)?;
                    // The next unvisited child moved into the same slot.
                    continue;
                }
                let run = self.normalize_run(doc, parent, next, &mut reached)?;
                let end = add(next, run.members.len())?;
                let source = add(run.source_bytes, mul(8, run.members.len())?)?;
                let plan = self.plan_dom_data(run.units(), run.units, source)?;
                self.work(8)?;
                doc.check_character_data_replacement(child, plan.stored_bytes())
                    .map_err(dom_data_error)?;
                let data = self.emit_dom_data(plan)?;
                self.work(8)?;
                doc.replace_character_data(child, data)
                    .map_err(dom_data_error)?;
                // Even a singleton uses the full canonical copy/replacement.
                // Removal is a separate reached phase after publication.
                if end > next_index {
                    self.normalize_remove(doc, parent, next_index, end, Some(child))?;
                }
                frames[frame_index].next = next_index;
            } else {
                frames[frame_index].next = next_index;
                if container && !doc.nodes[child].children.is_empty() {
                    self.normalize_push(
                        &mut frames,
                        NormalizeFrame {
                            parent: child,
                            next: 0,
                        },
                    )?;
                }
            }
        }
        Ok(Value::Undefined)
    }

    fn normalize_mark(&mut self, reached: &mut BTreeMap<NodeId, ()>, id: NodeId) -> Result<()> {
        self.work(16)?;
        let count = reached.len();
        if count >= crate::dom::MAX_NODES {
            return Err(overflow());
        }
        let mut height = 0;
        if count != 0 {
            height = 1;
            let mut capacity = add(count, 1)? / 2;
            while capacity >= 6 {
                capacity /= 6;
                height += 1;
            }
        }
        let levels = add(height, 1)?;
        let comparisons = count.min(mul(11, height)?);
        // One Entry search; conservative shifts/splits, new headers, and
        // eventual consuming teardown are all paid before touching the map.
        self.work(add(add(comparisons, mul(44, levels)?)?, 4)?)?;
        let node = add(
            mul(16, add(size_of::<NodeId>(), size_of::<()>())?)?,
            add(mul(32, size_of::<usize>())?, 64)?,
        )?;
        let nodes = if count == 0 {
            1
        } else if count < 11 {
            0
        } else {
            levels
        };
        self.charge(mul(nodes, node)?)?;
        // Logical admission does not turn BTreeMap's inherited infallible
        // allocation into a process-OOM recovery mechanism.
        match reached.entry(id) {
            Entry::Vacant(entry) => {
                entry.insert(());
                Ok(())
            }
            Entry::Occupied(_) => Err(invalid()),
        }
    }

    fn normalize_push(
        &mut self,
        frames: &mut Vec<NormalizeFrame>,
        frame: NormalizeFrame,
    ) -> Result<()> {
        self.work(1)?;
        let maximum = crate::dom::MAX_DEPTH + 1;
        if frames.len() >= maximum {
            return Err(overflow());
        }
        if frames.len() == frames.capacity() {
            let capacity = add(frames.len(), 1)?.max(mul(2, frames.capacity())?.min(maximum));
            self.work(add(1, mul(2, frames.len())?)?)?;
            self.charge(mul(capacity, size_of::<NormalizeFrame>())?)?;
            frames
                .try_reserve_exact(capacity - frames.len())
                .map_err(|_| ScriptError::resource("normalize cursor allocation failed"))?;
        }
        frames.push(frame);
        Ok(())
    }

    fn normalize_run<'d>(
        &mut self,
        doc: &'d Document,
        parent: NodeId,
        start: usize,
        reached: &mut BTreeMap<NodeId, ()>,
    ) -> Result<NormalizeRun<'d>> {
        // Parent/start were validated and the first member marked by the DFS.
        // All following Text members are checked/marked before any payload is
        // planned, so duplicate tails cannot change the current survivor.
        let children = &doc.nodes[parent].children;
        let mut end = add(start, 1)?;
        while end < children.len() {
            self.work(8)?;
            let id = children[end];
            let node = doc.nodes.get(id).ok_or_else(invalid)?;
            if node.parent != Some(parent) || id == doc.root {
                return Err(invalid());
            }
            if !matches!(node.kind, NodeKind::Text(_)) {
                break;
            }
            text_data(doc, id)?;
            self.normalize_mark(reached, id)?;
            end += 1;
        }
        let members = &children[start..end];
        let mut units = 0;
        let mut source_bytes = 0;
        for &id in members {
            self.work(8)?;
            let data = text_data(doc, id)?;
            self.work(add(1, data.stored_bytes())?)?;
            units = add(units, data.units().count())?;
            source_bytes = add(source_bytes, data.stored_bytes())?;
        }
        Ok(NormalizeRun {
            doc,
            members,
            units,
            source_bytes,
        })
    }

    fn normalize_remove(
        &mut self,
        doc: &mut Document,
        parent: NodeId,
        start: usize,
        end: usize,
        survivor: Option<NodeId>,
    ) -> Result<()> {
        let mut budget = DomMutationBudget {
            steps: self.steps,
            allocated: self.allocated,
            heap_limit: MAX_HEAP,
        };
        let result = doc.remove_normalize_text_range(parent, start, end, survivor, &mut budget);
        self.steps = budget.steps;
        self.allocated = budget.allocated;
        result.map_err(dom_data_error)
    }
}
