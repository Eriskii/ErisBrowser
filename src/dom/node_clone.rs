//! Checked detached cloning of the represented non-Document graph.
//! All Result failures precede publication, including details/task admission.
//! BTreeMap still allocates infallibly; this is not process-OOM recovery.
use super::mutation_budget::{add, map_nodes, movement, mul, push, search, tree};
use super::*;
use std::collections::btree_map::Entry;
use std::mem::size_of;

#[cfg(test)]
mod tests;

const PAGE_BITS: usize = 64;
const MAX_PAGES: usize = MAX_NODES.div_ceil(PAGE_BITS);

struct Frame {
    source: NodeId,
    copy: NodeId,
    parent: Option<NodeId>,
    root: NodeId,
    depth: usize,
    phase: u8,
    next: usize,
}

struct ClonePlan {
    nodes: Vec<Node>,
    groups: Vec<((NodeId, String), NodeId)>,
    tasks: Vec<(u64, DetailsToggle)>,
    retained: usize,
    name_bytes: usize,
    base_bytes: usize,
    sequence: u64,
}

fn invalid() -> DomDataError {
    DomDataError::InvalidNode
}

// Borrowed iterator construction and each next, including EOF, use the same
// pinned lazy-handle bound as Node equality. Empty maps take no iterator path.
fn iterator_work(count: usize) -> Result<usize, DomDataError> {
    add(8, mul(8, tree(count)?.1)?)
}

fn admit_map_insert<K, V>(
    count: usize,
    key_bytes: usize,
    budget: &mut DomMutationBudget,
) -> Result<(), DomDataError> {
    // Complete key comparison, four structural envelopes cover split-side and insertion-suffix repairs,
    // consuming cleanup, and header/entry setup even at count zero.
    budget.work(add(
        search(count, key_bytes)?,
        add(mul(4, movement(count)?)?, 32)?,
    )?)?;
    if count == 0 || count >= 11 {
        budget.charge(map_nodes::<K, V>(count)?)?;
    }
    Ok(())
}

fn staged_push<T>(
    values: &mut Vec<T>,
    value: T,
    maximum: usize,
    budget: &mut DomMutationBudget,
) -> Result<(), DomDataError> {
    budget.work(4)?; // eventual pop/consumption and buffer cleanup
    push(values, value, maximum, budget)
}

fn copy_string(value: &str, budget: &mut DomMutationBudget) -> Result<String, DomDataError> {
    budget.work(add(8, mul(2, value.len())?)?)?;
    budget.charge(value.len())?;
    let mut copy = String::new();
    copy.try_reserve_exact(value.len())
        .map_err(|_| DomDataError::AllocationFailed)?;
    copy.push_str(value);
    Ok(copy)
}

fn copy_data(value: &DomString, budget: &mut DomMutationBudget) -> Result<DomString, DomDataError> {
    budget.work(add(8, mul(2, value.stored_bytes())?)?)?;
    budget.charge(value.stored_bytes())?;
    value.try_clone_exact_owned()
}

fn copy_optional(
    value: &Option<String>,
    budget: &mut DomMutationBudget,
) -> Result<Option<String>, DomDataError> {
    budget.work(2)?;
    value.as_deref().map(|v| copy_string(v, budget)).transpose()
}

fn mark(
    pages: &mut BTreeMap<usize, u64>,
    id: NodeId,
    budget: &mut DomMutationBudget,
) -> Result<(), DomDataError> {
    budget.work(16)?;
    let count = pages.len();
    if id >= MAX_NODES || count > MAX_PAGES {
        return Err(invalid());
    }
    let page = id / PAGE_BITS;
    let mask = 1_u64 << (id % PAGE_BITS);
    let (comparisons, height) = tree(count)?;
    let levels = add(height, 1)?;
    budget.work(add(comparisons, add(mul(8, levels)?, 4)?)?)?;
    match pages.entry(page) {
        Entry::Occupied(mut entry) => {
            budget.work(8)?;
            if *entry.get() & mask != 0 {
                return Err(invalid());
            }
            *entry.get_mut() |= mask;
        }
        Entry::Vacant(entry) => {
            if count == MAX_PAGES {
                return Err(invalid());
            }
            // Same integer-pair split, backlink repair and consuming teardown
            // envelope as the independently reviewed equality page map.
            budget.work(add(mul(256, levels)?, 32)?)?;
            if count == 0 || count >= 11 {
                budget.charge(map_nodes::<usize, u64>(count)?)?;
            }
            entry.insert(mask);
        }
    }
    Ok(())
}

impl Document {
    pub(crate) fn clone_node_checked(
        &mut self,
        source: NodeId,
        deep: bool,
        budget: &mut DomMutationBudget,
    ) -> Result<NodeId, DomDataError> {
        budget.work(16)?;
        if self.nodes.len() > MAX_NODES
            || self.retained_bytes > MAX_DOM_BYTES
            || !matches!(
                self.nodes.get(self.root),
                Some(Node {
                    kind: NodeKind::Document,
                    parent: None,
                    ..
                })
            )
        {
            return Err(invalid());
        }
        let plan = self.prepare_clone(source, deep, budget)?;
        let root = self.nodes.len();
        let count = plan.nodes.len();
        let required = add(root, count)?;
        if required > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        budget.charge(mul(128, count)?)?;
        budget.work(add(
            8,
            add(
                mul(2, count)?,
                add(mul(6, plan.groups.len())?, mul(6, plan.tasks.len())?)?,
            )?,
        )?)?;
        if count > self.nodes.capacity().saturating_sub(root) {
            budget.work(add(1, mul(2, root)?)?)?;
            budget.charge(mul(required, size_of::<Node>())?)?;
            // Final fallible operation. No Result, callback or budget decision
            // follows it. Metadata BTree allocations remain infallible.
            self.nodes
                .try_reserve_exact(count)
                .map_err(|_| DomDataError::AllocationFailed)?;
        }
        for node in plan.nodes {
            self.nodes.push(node);
        }
        for (key, node) in plan.groups {
            self.details_groups.insert(key, node);
        }
        for (sequence, event) in plan.tasks {
            self.details_trackers.insert(
                event.node,
                DetailsToggleTracker {
                    task: sequence,
                    old_open: event.old_open,
                },
            );
            self.details_toggles.insert(sequence, event);
        }
        self.retained_bytes = plan.retained;
        self.details_name_bytes = plan.name_bytes;
        self.base_href_bytes = plan.base_bytes;
        self.details_sequence = plan.sequence;
        Ok(root)
    }

    fn prepare_clone(
        &self,
        source: NodeId,
        deep: bool,
        budget: &mut DomMutationBudget,
    ) -> Result<ClonePlan, DomDataError> {
        budget.work(16)?;
        let node = self.nodes.get(source).ok_or_else(invalid)?;
        let graph = deep
            && (!node.children.is_empty()
                || matches!(&node.kind,
            NodeKind::Element(el) if el.template_contents.is_some()));
        let mut pages = BTreeMap::new();
        let mut frames = Vec::new();
        let mut winners = BTreeMap::new();
        let mut plan = ClonePlan {
            nodes: Vec::new(),
            groups: Vec::new(),
            tasks: Vec::new(),
            retained: 0,
            name_bytes: 0,
            base_bytes: 0,
            sequence: self.details_sequence,
        };
        let copy =
            self.clone_single(source, deep, 0, None, &mut plan, graph, &mut pages, budget)?;
        let first = Frame {
            source,
            copy,
            parent: None,
            root: copy,
            depth: 0,
            phase: 0,
            next: 0,
        };
        // A shallow ordinary node/leaf has no DFS or visited-map allocation.
        if !graph {
            self.clone_details(source, copy, copy, &mut plan, &mut winners, budget)?;
        } else {
            staged_push(&mut frames, first, MAX_DEPTH + 1, budget)?;
            while let Some(frame) = frames.last_mut() {
                budget.work(8)?;
                if frame.phase == 0 {
                    frame.phase = 1;
                    if let NodeKind::Element(el) = &self.nodes[frame.source].kind
                        && let Some(content) = el.template_contents
                    {
                        let NodeKind::Element(copy_element) =
                            &plan.nodes[frame.copy - self.nodes.len()].kind
                        else {
                            return Err(invalid());
                        };
                        let content_copy = copy_element.template_contents.ok_or_else(invalid)?;
                        let child_frame = Frame {
                            source: content,
                            copy: content_copy,
                            parent: None,
                            root: content_copy,
                            depth: add(frame.depth, 1)?,
                            phase: 1,
                            next: 0,
                        };
                        staged_push(&mut frames, child_frame, MAX_DEPTH + 1, budget)?;
                    }
                    continue;
                }
                if frame.phase == 1 {
                    frame.phase = 2;
                    budget.work(8)?;
                    if let Some(parent) = frame.parent {
                        plan.nodes[frame.copy - self.nodes.len()].parent = Some(parent);
                        plan.nodes[parent - self.nodes.len()]
                            .children
                            .push(frame.copy);
                    }
                    self.clone_details(
                        frame.source,
                        frame.copy,
                        frame.root,
                        &mut plan,
                        &mut winners,
                        budget,
                    )?;
                    continue;
                }
                if let Some(&child) = self.nodes[frame.source].children.get(frame.next) {
                    budget.work(16)?;
                    frame.next += 1;
                    let node = self.nodes.get(child).ok_or_else(invalid)?;
                    if node.parent != Some(frame.source)
                        || matches!(
                            node.kind,
                            NodeKind::Document
                                | NodeKind::DocumentFragment { .. }
                                | NodeKind::Doctype(_)
                        )
                    {
                        return Err(invalid());
                    }
                    let depth = add(frame.depth, 1)?;
                    let parent = frame.copy;
                    let ordinary_root = frame.root;
                    let copy = self.clone_single(
                        child, true, depth, None, &mut plan, true, &mut pages, budget,
                    )?;
                    staged_push(
                        &mut frames,
                        Frame {
                            source: child,
                            copy,
                            parent: Some(parent),
                            root: ordinary_root,
                            depth,
                            phase: 0,
                            next: 0,
                        },
                        MAX_DEPTH + 1,
                        budget,
                    )?;
                } else {
                    frames.pop();
                }
            }
        }
        plan.retained = add(self.retained_bytes, plan.retained)?;
        if plan.retained > MAX_DOM_BYTES {
            return Err(DomDataError::LimitExceeded);
        }
        plan.name_bytes = add(self.details_name_bytes, plan.name_bytes)?;
        plan.base_bytes = add(self.base_href_bytes, plan.base_bytes)?;
        self.admit_clone_metadata(&plan, budget)?;
        Ok(plan)
    }

    #[allow(clippy::too_many_arguments)]
    fn clone_single(
        &self,
        source: NodeId,
        deep: bool,
        depth: usize,
        host: Option<NodeId>,
        plan: &mut ClonePlan,
        graph: bool,
        pages: &mut BTreeMap<usize, u64>,
        budget: &mut DomMutationBudget,
    ) -> Result<NodeId, DomDataError> {
        budget.work(32)?;
        if depth > MAX_DEPTH || add(self.nodes.len(), plan.nodes.len())? >= MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        let node = self.nodes.get(source).ok_or_else(invalid)?;
        if !node.children.is_empty()
            && !matches!(
                node.kind,
                NodeKind::Element(_) | NodeKind::DocumentFragment { .. }
            )
        {
            return Err(invalid());
        }
        if graph {
            mark(pages, source, budget)?;
        }
        let mut content = None;
        let (kind, bytes) = match &node.kind {
            NodeKind::Document => return Err(invalid()),
            NodeKind::DocumentFragment { host: old_host } => {
                if node.parent.is_some() || old_host.is_some_and(|id| !matches!(self.nodes.get(id).map(|n| &n.kind),
                    Some(NodeKind::Element(el)) if el.namespace == Namespace::Html && el.tag == "template" && el.template_contents == Some(source)))
                {
                    return Err(invalid());
                }
                (NodeKind::DocumentFragment { host }, 0)
            }
            NodeKind::Text(value) => (
                NodeKind::Text(copy_data(value, budget)?),
                value.stored_bytes(),
            ),
            NodeKind::Comment(value) => (
                NodeKind::Comment(copy_data(value, budget)?),
                value.stored_bytes(),
            ),
            NodeKind::ProcessingInstruction { target, data } => (
                NodeKind::ProcessingInstruction {
                    target: copy_string(target, budget)?,
                    data: copy_data(data, budget)?,
                },
                add(target.len(), data.stored_bytes())?,
            ),
            NodeKind::Doctype(value) => (
                NodeKind::Doctype(Doctype {
                    name: copy_string(&value.name, budget)?,
                    public_id: copy_optional(&value.public_id, budget)?,
                    system_id: copy_optional(&value.system_id, budget)?,
                    force_quirks: value.force_quirks,
                }),
                add(
                    value.name.len(),
                    add(
                        value.public_id.as_ref().map_or(0, String::len),
                        value.system_id.as_ref().map_or(0, String::len),
                    )?,
                )?,
            ),
            NodeKind::Element(el) => {
                if el.attrs.len() > 1024
                    || el.attr_namespaces.len() > 11
                    || el.attr_namespaces.len() > el.attrs.len()
                {
                    return Err(invalid());
                }
                if el.namespace == Namespace::Html && el.tag == "template" {
                    let id = el.template_contents.ok_or_else(invalid)?;
                    if id == source
                        || !matches!(self.nodes.get(id), Some(Node { parent: None,
                        kind: NodeKind::DocumentFragment { host: Some(owner) }, .. }) if *owner == source)
                    {
                        return Err(invalid());
                    }
                    content = Some(id);
                } else if el.template_contents.is_some() {
                    return Err(invalid());
                }
                let mut copied = Element {
                    namespace: el.namespace,
                    tag: copy_string(&el.tag, budget)?,
                    attrs: BTreeMap::new(),
                    attr_namespaces: BTreeMap::new(),
                    template_contents: None,
                };
                let mut bytes = el.tag.len();
                if !el.attrs.is_empty() {
                    let work = iterator_work(el.attrs.len())?;
                    budget.work(work)?;
                    let mut entries = el.attrs.iter();
                    loop {
                        budget.work(work)?;
                        let Some((key, value)) = entries.next() else {
                            break;
                        };
                        bytes = add(bytes, add(key.len(), value.len())?)?;
                        let key = copy_string(key, budget)?;
                        let value = copy_string(value, budget)?;
                        admit_map_insert::<String, String>(copied.attrs.len(), key.len(), budget)?;
                        copied.attrs.insert(key, value);
                    }
                }
                if !el.attr_namespaces.is_empty() {
                    let work = iterator_work(el.attr_namespaces.len())?;
                    budget.work(work)?;
                    let mut entries = el.attr_namespaces.iter();
                    loop {
                        budget.work(work)?;
                        let Some((key, namespace)) = entries.next() else {
                            break;
                        };
                        budget.work(add(32 + 11 * 14, search(el.attrs.len(), key.len())?)?)?;
                        if key.len() > 13
                            || AttributeNamespace::from_qualified_name(key) != Some(*namespace)
                            || !el.attrs.contains_key(key)
                        {
                            return Err(invalid());
                        }
                        bytes = add(bytes, key.len())?;
                        let key = copy_string(key, budget)?;
                        admit_map_insert::<String, AttributeNamespace>(
                            copied.attr_namespaces.len(),
                            key.len(),
                            budget,
                        )?;
                        copied.attr_namespaces.insert(key, *namespace);
                    }
                }
                (NodeKind::Element(copied), bytes)
            }
        };
        plan.retained = add(plan.retained, bytes)?;
        let count = if deep { node.children.len() } else { 0 };
        if count > MAX_NODES.saturating_sub(self.nodes.len()) {
            return Err(DomDataError::LimitExceeded);
        }
        budget.work(8)?; // reservation/header and eventual child-vector cleanup
        budget.charge(mul(count, size_of::<NodeId>())?)?;
        let mut children = Vec::new();
        children
            .try_reserve_exact(count)
            .map_err(|_| DomDataError::AllocationFailed)?;
        let id = add(self.nodes.len(), plan.nodes.len())?;
        staged_push(
            &mut plan.nodes,
            Node {
                parent: None,
                children,
                kind,
            },
            MAX_NODES - self.nodes.len(),
            budget,
        )?;
        if let Some(content) = content {
            let copy = self.clone_single(
                content,
                deep,
                add(depth, 1)?,
                Some(id),
                plan,
                graph,
                pages,
                budget,
            )?;
            if let NodeKind::Element(el) = &mut plan.nodes[id - self.nodes.len()].kind {
                el.template_contents = Some(copy);
            }
        }
        Ok(id)
    }

    fn clone_details<'a>(
        &'a self,
        source: NodeId,
        copy: NodeId,
        root: NodeId,
        plan: &mut ClonePlan,
        winners: &mut BTreeMap<(NodeId, &'a str), NodeId>,
        budget: &mut DomMutationBudget,
    ) -> Result<(), DomDataError> {
        budget.work(16)?;
        let NodeKind::Element(el) = &self.nodes[source].kind else {
            return Ok(());
        };
        if el.namespace != Namespace::Html {
            return Ok(());
        }
        if el.tag == "base" {
            budget.work(add(8, search(el.attrs.len(), 4)?)?)?;
            plan.base_bytes = add(plan.base_bytes, el.attrs.get("href").map_or(0, String::len))?;
        }
        if el.tag != "details" {
            return Ok(());
        }
        budget.work(add(16, mul(2, search(el.attrs.len(), 4)?)?)?)?;
        let name = el.attrs.get("name").map(String::as_str).unwrap_or("");
        plan.name_bytes = add(plan.name_bytes, name.len())?;
        if !el.attrs.contains_key("open") {
            return Ok(());
        }
        let mut sequence = plan.sequence;
        plan.sequence = plan
            .sequence
            .checked_add(1)
            .ok_or(DomDataError::LimitExceeded)?;
        let mut new_open = true;
        if !name.is_empty() {
            let count = winners.len();
            budget.work(add(8, search(count, name.len())?)?)?;
            match winners.entry((root, name)) {
                Entry::Occupied(_) => {
                    let NodeKind::Element(copy_el) = &mut plan.nodes[copy - self.nodes.len()].kind
                    else {
                        return Err(invalid());
                    };
                    budget.work(add(
                        search(copy_el.attrs.len(), 4)?,
                        add(mul(4, movement(copy_el.attrs.len())?)?, 16)?,
                    )?)?;
                    let value = copy_el.attrs.remove("open").ok_or_else(invalid)?;
                    plan.retained = plan
                        .retained
                        .checked_sub(add(4, value.len())?)
                        .ok_or_else(invalid)?;
                    sequence = plan.sequence;
                    plan.sequence = plan
                        .sequence
                        .checked_add(1)
                        .ok_or(DomDataError::LimitExceeded)?;
                    new_open = false;
                }
                Entry::Vacant(entry) => {
                    admit_map_insert::<(NodeId, &str), NodeId>(count, name.len(), budget)?;
                    let key = (root, copy_string(name, budget)?);
                    staged_push(&mut plan.groups, (key, copy), MAX_NODES, budget)?;
                    entry.insert(copy);
                }
            }
        }
        budget.work(16)?; // task/tracker fields and conceptual event counters
        staged_push(
            &mut plan.tasks,
            (
                sequence,
                DetailsToggle {
                    node: copy,
                    old_open: false,
                    new_open,
                },
            ),
            MAX_NODES,
            budget,
        )
    }

    fn admit_clone_metadata(
        &self,
        plan: &ClonePlan,
        budget: &mut DomMutationBudget,
    ) -> Result<(), DomDataError> {
        budget.work(8)?;
        for (index, (key, _)) in plan.groups.iter().enumerate() {
            budget.work(add(8, search(self.details_groups.len(), key.1.len())?)?)?;
            if self.details_groups.contains_key(key) {
                return Err(invalid());
            }
            admit_map_insert::<(NodeId, String), NodeId>(
                add(self.details_groups.len(), index)?,
                key.1.len(),
                budget,
            )?;
        }
        for (index, (sequence, event)) in plan.tasks.iter().enumerate() {
            budget.work(add(
                16,
                add(
                    search(self.details_trackers.len(), 0)?,
                    search(self.details_toggles.len(), 0)?,
                )?,
            )?)?;
            if self.details_trackers.contains_key(&event.node)
                || self.details_toggles.contains_key(sequence)
                || self.details_active.is_some_and(|task| task.id == *sequence)
            {
                return Err(invalid());
            }
            admit_map_insert::<NodeId, DetailsToggleTracker>(
                add(self.details_trackers.len(), index)?,
                0,
                budget,
            )?;
            admit_map_insert::<u64, DetailsToggle>(
                add(self.details_toggles.len(), index)?,
                0,
                budget,
            )?;
        }
        Ok(())
    }
}
