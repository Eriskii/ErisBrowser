//! Checked script-facing replacement of an Element/Fragment's direct children.
//!
//! Every budget decision and fallible allocation precedes mutation. BTreeMap
//! and the existing URL/encoding/IDNA implementation still allocate infallibly;
//! this is not a process-OOM recovery mechanism.
use super::*;
use std::mem::size_of;

use super::mutation_budget::{add, map_nodes, movement, mul, push, search};

struct MovingDetails {
    key: (NodeId, String),
    node: NodeId,
    new_root: NodeId,
    attributes: usize,
    namespaces: usize,
}

struct Replacement {
    data: Option<DomString>,
    children: Vec<NodeId>,
    details: Vec<MovingDetails>,
    // None preserves the old frozen URL; Some(None) removes the selected base.
    base: Option<Option<(NodeId, url::Url)>>,
    retained: usize,
}

impl Document {
    /// O(1) pre-emission check. Runtime pays its fixed admission debit first.
    /// Detached old children remain retained and provide no byte/node credit.
    pub(crate) fn check_text_content_replacement(
        &self,
        parent: NodeId,
        stored_bytes: usize,
    ) -> Result<(), DomDataError> {
        if !matches!(
            self.nodes.get(parent).map(|node| &node.kind),
            Some(NodeKind::Element(_) | NodeKind::DocumentFragment { .. })
        ) {
            return Err(DomDataError::InvalidNode);
        }
        if stored_bytes != 0 && !self.admits_text_node(stored_bytes) {
            return Err(DomDataError::LimitExceeded);
        }
        Ok(())
    }

    pub(crate) fn replace_text_content_owned(
        &mut self,
        parent: NodeId,
        data: DomString,
        budget: &mut DomMutationBudget,
    ) -> Result<Option<NodeId>, DomDataError> {
        budget.work(8)?;
        self.check_text_content_replacement(parent, data.stored_bytes())?;
        let prepared = self.prepare_text_replacement(parent, data, budget)?;

        // Every quota decision is before the final fallible arena reserve.
        // These are the existing script logical-node and actual growth fees.
        if prepared.data.is_some() {
            budget.charge(128)?;
            if self.nodes.len() == self.nodes.capacity() {
                let required = add(self.nodes.len(), 1)?;
                budget.work(add(1, mul(self.nodes.len(), 2)?)?)?;
                budget.charge(mul(required, size_of::<Node>())?)?;
                self.nodes
                    .try_reserve_exact(1)
                    .map_err(|_| DomDataError::AllocationFailed)?;
            }
        }

        // No fallible operation, callback, or Resource decision below here.
        Ok(self.commit_text_replacement(parent, prepared))
    }

    fn prepare_text_replacement(
        &self,
        parent: NodeId,
        data: DomString,
        budget: &mut DomMutationBudget,
    ) -> Result<Replacement, DomDataError> {
        let nonempty = data.stored_bytes() != 0;
        let retained = add(self.retained_bytes, data.stored_bytes())?;
        let mut children = Vec::new();
        if nonempty {
            let mut cursor = Some(parent);
            let mut ancestors = 0;
            while let Some(id) = cursor {
                budget.work(1)?;
                if ancestors >= MAX_DEPTH {
                    return Err(DomDataError::LimitExceeded);
                }
                let node = self.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
                ancestors += 1;
                cursor = node.parent.or(match node.kind {
                    NodeKind::DocumentFragment { host } => host,
                    _ => None,
                });
            }
            if add(ancestors, 1)? > MAX_DEPTH {
                return Err(DomDataError::LimitExceeded);
            }
            budget.charge(size_of::<NodeId>())?;
            children
                .try_reserve_exact(1)
                .map_err(|_| DomDataError::AllocationFailed)?;
            budget.work(1)?;
            children.push(self.nodes.len());
        }
        let old_children = &self.nodes[parent].children;
        for &child in old_children {
            budget.work(2)?;
            let node = self.nodes.get(child).ok_or(DomDataError::InvalidNode)?;
            if child == parent || child == self.root || node.parent != Some(parent) {
                return Err(DomDataError::InvalidNode);
            }
        }
        let details = self.prepare_detached_details(parent, budget)?;
        let base = self.prepare_cleared_base(parent, budget)?;
        self.charge_details_commit(&details, budget)?;
        let summaries = self.details_summaries.borrow().len();
        budget.work(add(search(summaries, 0)?, movement(summaries)?)?)?;
        budget.work(add(8, mul(old_children.len(), 2)?)?)?;
        Ok(Replacement {
            data: nonempty.then_some(data),
            children,
            details,
            base,
            retained,
        })
    }

    fn prepare_detached_details(
        &self,
        parent: NodeId,
        budget: &mut DomMutationBudget,
    ) -> Result<Vec<MovingDetails>, DomDataError> {
        let mut rows = Vec::new();
        if self.details_groups.is_empty() || self.nodes[parent].children.is_empty() {
            return Ok(rows);
        }
        let mut old_root = parent;
        let mut ancestors = 0;
        loop {
            budget.work(1)?;
            let node = self.nodes.get(old_root).ok_or(DomDataError::InvalidNode)?;
            let Some(next) = node.parent else { break };
            if ancestors >= MAX_DEPTH {
                return Err(DomDataError::LimitExceeded);
            }
            old_root = next;
            ancestors += 1;
        }
        let mut pending = Vec::new();
        let mut visited = 0;
        let mut possible_removed = 0;
        for &root in &self.nodes[parent].children {
            push(&mut pending, (root, 1usize), self.nodes.len(), budget)?;
            while let Some((id, depth)) = pending.pop() {
                budget.work(8)?;
                visited = add(visited, 1)?;
                if visited > self.nodes.len().min(MAX_NODES) || depth > MAX_DEPTH {
                    return Err(DomDataError::LimitExceeded);
                }
                let node = self.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
                if let NodeKind::Element(element) = &node.kind
                    && element.namespace == Namespace::Html
                    && element.tag == "details"
                {
                    budget.work(search(element.attrs.len(), 4)?)?;
                    if let Some(open) = element.attrs.get("open") {
                        budget.work(search(element.attrs.len(), 4)?)?;
                        if let Some(name) = element.attrs.get("name").filter(|s| !s.is_empty()) {
                            budget.work(search(element.attr_namespaces.len(), 4)?)?;
                            let removed = add(
                                add(4, open.len())?,
                                if element.attr_namespaces.contains_key("open") {
                                    4
                                } else {
                                    0
                                },
                            )?;
                            possible_removed = add(possible_removed, removed)?;
                            if possible_removed > self.retained_bytes {
                                return Err(DomDataError::InvalidNode);
                            }
                            budget.work(name.len())?;
                            budget.charge(name.len())?;
                            let mut copied = String::new();
                            copied
                                .try_reserve_exact(name.len())
                                .map_err(|_| DomDataError::AllocationFailed)?;
                            copied.push_str(name);
                            push(
                                &mut rows,
                                MovingDetails {
                                    key: (old_root, copied),
                                    node: id,
                                    new_root: root,
                                    attributes: element.attrs.len(),
                                    namespaces: element.attr_namespaces.len(),
                                },
                                self.nodes.len(),
                                budget,
                            )?;
                        }
                    }
                }
                // Hosted template contents retain their separate root/group.
                for &child in node.children.iter().rev() {
                    budget.work(2)?;
                    if self.nodes.get(child).and_then(|n| n.parent) != Some(id) {
                        return Err(DomDataError::InvalidNode);
                    }
                    push(
                        &mut pending,
                        (child, add(depth, 1)?),
                        self.nodes.len(),
                        budget,
                    )?;
                }
            }
        }
        Ok(rows)
    }

    fn charge_details_commit(
        &self,
        rows: &[MovingDetails],
        budget: &mut DomMutationBudget,
    ) -> Result<(), DomDataError> {
        let groups = add(self.details_groups.len(), rows.len())?;
        let trackers = add(self.details_trackers.len(), rows.len())?;
        let toggles = add(self.details_toggles.len(), rows.len())?;
        for row in rows {
            budget.work(add(
                mul(4, search(groups, row.key.1.len())?)?,
                mul(2, movement(groups)?)?,
            )?)?;
            budget.charge(map_nodes::<(NodeId, String), NodeId>(groups)?)?;
            for count in [trackers, toggles] {
                budget.work(add(mul(2, search(count, 0)?)?, mul(2, movement(count)?)?)?)?;
            }
            budget.charge(add(
                map_nodes::<NodeId, DetailsToggleTracker>(trackers)?,
                map_nodes::<u64, DetailsToggle>(toggles)?,
            )?)?;
            for count in [row.attributes, row.namespaces] {
                budget.work(add(search(count, 4)?, movement(count)?)?)?;
            }
            budget.work(24)?;
        }
        Ok(())
    }

    fn prepare_cleared_base(
        &self,
        parent: NodeId,
        budget: &mut DomMutationBudget,
    ) -> Result<Option<Option<(NodeId, url::Url)>>, DomDataError> {
        if !self.base_tracking {
            return Ok(None);
        }
        let Some((selected, _)) = self.first_base.as_ref() else {
            return Ok(None);
        };
        if *selected == parent {
            return Ok(None);
        }
        let mut cursor = Some(*selected);
        let mut affected = false;
        for _ in 0..=MAX_DEPTH {
            let Some(id) = cursor else { break };
            budget.work(1)?;
            if id == parent {
                affected = true;
                break;
            }
            cursor = self.nodes.get(id).ok_or(DomDataError::InvalidNode)?.parent;
        }
        if !affected {
            if cursor.is_some() {
                return Err(DomDataError::LimitExceeded);
            }
            return Ok(None);
        }
        // Preserve the inherited URL/traversal debit; explicit stack, edge and
        // attribute bookkeeping below is additional work actually performed.
        budget.work(add(
            add(self.nodes.len(), self.base_href_bytes)?,
            add(self.url.as_str().len(), 1)?,
        )?)?;
        let mut pending = Vec::new();
        push(&mut pending, (self.root, 0usize), self.nodes.len(), budget)?;
        let mut visited = 0;
        while let Some((id, depth)) = pending.pop() {
            visited = add(visited, 1)?;
            if visited > self.nodes.len().min(MAX_NODES) || depth > MAX_DEPTH {
                return Err(DomDataError::LimitExceeded);
            }
            let node = self.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
            if let NodeKind::Element(element) = &node.kind
                && element.namespace == Namespace::Html
                && element.tag == "base"
            {
                budget.work(search(element.attrs.len(), 4)?)?;
                if let Some(href) = element.attrs.get("href") {
                    if id == *selected {
                        return Ok(None);
                    }
                    // The existing URL/IDNA/encoding internals allocate
                    // infallibly and do not expose an exact allocator meter.
                    // Resolve entirely before mutation, then admit the retained
                    // result; the inherited limitation remains explicit.
                    let url = crate::document_url::parse(self, &self.url, href)
                        .ok()
                        .filter(|url| {
                            !matches!(url.scheme(), "data" | "javascript")
                                && url.as_str().len() <= MAX_DOM_BYTES
                        })
                        .unwrap_or_else(|| self.url.clone());
                    budget.charge(url.as_str().len())?;
                    return Ok(Some(Some((id, url))));
                }
            }
            if id != parent {
                for &child in node.children.iter().rev() {
                    budget.work(2)?;
                    if self.nodes.get(child).and_then(|n| n.parent) != Some(id) {
                        return Err(DomDataError::InvalidNode);
                    }
                    push(
                        &mut pending,
                        (child, add(depth, 1)?),
                        self.nodes.len(),
                        budget,
                    )?;
                }
            }
        }
        Ok(Some(None))
    }

    fn commit_text_replacement(&mut self, parent: NodeId, prepared: Replacement) -> Option<NodeId> {
        for row in &prepared.details {
            if self.details_groups.get(&row.key) == Some(&row.node) {
                self.details_groups.remove(&row.key);
            }
        }
        self.details_summaries.get_mut().remove(&parent);
        for child in std::mem::take(&mut self.nodes[parent].children) {
            self.nodes[child].parent = None;
        }
        self.retained_bytes = prepared.retained;
        for mut row in prepared.details {
            row.key.0 = row.new_root;
            if self
                .details_groups
                .get(&row.key)
                .is_some_and(|&winner| winner != row.node)
            {
                // Equivalent to enforce_details_group(false)'s open removal,
                // without name normalization/copy or a post-mutation debit.
                let NodeKind::Element(element) = &mut self.nodes[row.node].kind else {
                    unreachable!()
                };
                if let Some(value) = element.attrs.remove("open") {
                    self.retained_bytes -= 4 + value.len();
                    if element.attr_namespaces.remove("open").is_some() {
                        self.retained_bytes -= 4;
                    }
                    self.queue_details_toggle(row.node, true, false);
                }
            } else {
                self.details_groups.insert(row.key, row.node);
            }
        }
        if let Some(base) = prepared.base {
            self.first_base = base;
        }
        let result = prepared.data.map(|data| {
            let id = self.nodes.len();
            self.nodes.push(Node {
                parent: Some(parent),
                children: Vec::new(),
                kind: NodeKind::Text(data),
            });
            id
        });
        self.nodes[parent].children = prepared.children;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget() -> DomMutationBudget {
        DomMutationBudget {
            steps: 100_000,
            allocated: 0,
            heap_limit: 8 * 1024 * 1024,
        }
    }

    fn basic() -> (Document, NodeId) {
        let document = Document::parse("<div id=parent><span>retained</span><!--comment--></div>");
        let parent = document.query_selector("#parent").unwrap();
        (document, parent)
    }

    fn hooks() -> (Document, NodeId) {
        let (mut document, parent) = basic();
        document.initialize_url(url::Url::parse("https://example.test/original/").unwrap());
        let branch = document.nodes[parent].children[0];
        let details = document.create_element("details");
        document.set_attr(details, "name", "named");
        document.set_attr(details, "open", "kept");
        document.append_child(branch, details);
        let first = document.create_element("base");
        document.set_attr(first, "href", "https://first.test/");
        document.append_child(branch, first);
        let later = document.create_element("base");
        document.set_attr(later, "href", "https://later.test/");
        let body = document.query_selector("body").unwrap();
        document.append_child(body, later);
        document.first_summary(details);
        (document, parent)
    }

    fn snapshot(document: &Document) -> String {
        format!("{document:?}")
    }

    #[test]
    fn replacement_moves_exact_payload_and_retains_old_branch_identity() {
        let (mut document, parent) = basic();
        let before = document.nodes.len();
        let retained = document.retained_bytes;
        let old = document.nodes[parent].children.clone();
        let branch = old[0];
        let descendant = document.nodes[branch].children[0];
        let data = DomString::from_nonscalar_units(vec![0xd800, 65, 0xdc00]).unwrap();
        let pointer = data.raw_units().unwrap().as_ptr();
        let new = document
            .replace_text_content_owned(parent, data, &mut budget())
            .unwrap()
            .unwrap();
        assert_eq!(new, before);
        assert_eq!(document.nodes[parent].children, [new]);
        assert_eq!(document.nodes[new].parent, Some(parent));
        let NodeKind::Text(text) = &document.nodes[new].kind else {
            panic!()
        };
        assert_eq!(text.raw_units().unwrap(), [0xd800, 65, 0xdc00]);
        assert_eq!(text.raw_units().unwrap().as_ptr(), pointer);
        assert_eq!(document.retained_bytes, retained + 6);
        for old in old {
            assert_eq!(document.nodes[old].parent, None);
        }
        assert_eq!(document.nodes[descendant].parent, Some(branch));
        assert_eq!(document.text_content(descendant), "retained");
    }

    #[test]
    fn same_text_still_creates_a_new_node_and_empty_at_caps_creates_none() {
        let mut document = Document::parse("<div id=parent>same</div>");
        let parent = document.query_selector("#parent").unwrap();
        let old = document.nodes[parent].children[0];
        let new = document
            .replace_text_content_owned(parent, "same".into(), &mut budget())
            .unwrap()
            .unwrap();
        assert_ne!(new, old);
        assert_eq!(document.nodes[old].parent, None);
        while document.nodes.len() < MAX_NODES {
            document.nodes.push(Node {
                parent: None,
                children: Vec::new(),
                kind: NodeKind::Comment("".into()),
            });
        }
        document.retained_bytes = MAX_DOM_BYTES;
        assert_eq!(
            document.check_text_content_replacement(parent, 1),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(document.check_text_content_replacement(parent, 0), Ok(()));
        assert_eq!(
            document.replace_text_content_owned(parent, "".into(), &mut budget()),
            Ok(None)
        );
        assert!(document.nodes[parent].children.is_empty());
        assert_eq!(document.nodes[new].parent, None);
        assert_eq!(document.nodes.len(), MAX_NODES);
        assert_eq!(document.retained_bytes, MAX_DOM_BYTES);
    }

    #[test]
    fn preemission_admission_has_no_detached_storage_credit() {
        let (mut document, parent) = basic();
        document.retained_bytes = MAX_DOM_BYTES - 2;
        let before = snapshot(&document);
        assert_eq!(document.check_text_content_replacement(parent, 2), Ok(()));
        assert_eq!(
            document.check_text_content_replacement(parent, 3),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(
            document.check_text_content_replacement(parent, usize::MAX),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(
            document.check_text_content_replacement(document.root, 0),
            Err(DomDataError::InvalidNode)
        );
        assert_eq!(
            document.check_text_content_replacement(usize::MAX, 0),
            Err(DomDataError::InvalidNode)
        );
        assert_eq!(snapshot(&document), before);
    }

    #[test]
    fn all_work_and_heap_cutpoints_leave_outer_tree_and_hooks_unchanged() {
        for use_hooks in [false, true] {
            for growth in [false, true] {
                let fresh = || {
                    let (mut document, parent) = if use_hooks { hooks() } else { basic() };
                    document.nodes.shrink_to_fit();
                    if !growth {
                        document.nodes.reserve_exact(1);
                    }
                    (document, parent)
                };
                for units in [false, true] {
                    let data = || {
                        if units {
                            DomString::from_nonscalar_units(vec![0xd800, 65]).unwrap()
                        } else {
                            "new".into()
                        }
                    };
                    let (mut measured, parent) = fresh();
                    let mut ample = budget();
                    measured
                        .replace_text_content_owned(parent, data(), &mut ample)
                        .unwrap();
                    let work = 100_000 - ample.steps;
                    let heap = ample.allocated;
                    assert!(work > 0 && heap > 0);
                    for (steps, heap_limit, succeeds) in [
                        (work, heap, true),
                        (work - 1, heap, false),
                        (work, heap - 1, false),
                    ] {
                        let (mut document, parent) = fresh();
                        let before = snapshot(&document);
                        let capacity = document.nodes.capacity();
                        let mut meter = DomMutationBudget {
                            steps,
                            allocated: 0,
                            heap_limit,
                        };
                        let result =
                            document.replace_text_content_owned(parent, data(), &mut meter);
                        assert_eq!(
                            result.is_ok(),
                            succeeds,
                            "hooks={use_hooks}, growth={growth}, units={units}"
                        );
                        if !succeeds {
                            assert_eq!(snapshot(&document), before);
                            assert_eq!(document.nodes.capacity(), capacity);
                            if steps < work {
                                assert_eq!(meter.steps, 0);
                            }
                            if heap_limit < heap {
                                assert!(meter.allocated > heap_limit);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn details_root_change_and_base_selection_match_existing_clear() {
        let (mut document, parent) = hooks();
        let mut expected = document.clone();
        expected.clear_children(parent);
        assert_eq!(
            document.replace_text_content_owned(parent, "".into(), &mut budget()),
            Ok(None)
        );
        assert_eq!(snapshot(&document), snapshot(&expected));
        assert_eq!(document.base_url().as_str(), "https://later.test/");
        let detached = document
            .nodes
            .iter()
            .enumerate()
            .find_map(|(id, node)| {
                matches!(&node.kind, NodeKind::Element(el) if el.tag == "details").then_some(id)
            })
            .unwrap();
        assert_eq!(document.attr(detached, "open"), Some("kept"));
        let peer = document.create_element("details");
        document.set_attr(peer, "name", "named");
        let body = document.query_selector("body").unwrap();
        document.append_child(body, peer);
        document.set_attr(peer, "open", "");
        assert_eq!(document.attr(detached, "open"), Some("kept"));
    }

    #[test]
    fn conflict_closure_preserves_active_toggle_coalescing_and_byte_debits() {
        let (mut document, parent) = hooks();
        let branch = document.nodes[parent].children[0];
        let details = document.nodes[branch]
            .children
            .iter()
            .copied()
            .find(|&id| document.tag(id) == Some("details"))
            .unwrap();
        let other = document.create_element("details");
        document
            .details_groups
            .insert((branch, "named".into()), other);
        let queued = document.peek_details_toggle().unwrap();
        document.begin_details_toggle(queued.id).unwrap();
        let mut expected = document.clone();
        expected.clear_children(parent);
        document
            .replace_text_content_owned(parent, "".into(), &mut budget())
            .unwrap();
        assert_eq!(snapshot(&document), snapshot(&expected));
        assert!(document.attr(details, "open").is_none());
        assert!(document.details_active.is_some());
    }

    #[test]
    fn unaffected_base_keeps_frozen_url_and_affected_invalid_base_falls_back() {
        let (mut document, parent) = hooks();
        let first = document.frozen_base().unwrap().0;
        let frozen = document.base_url().clone();
        document.set_url(url::Url::parse("https://example.test/changed/").unwrap());
        document
            .replace_text_content_owned(first, "base child".into(), &mut budget())
            .unwrap();
        assert_eq!(document.base_url(), &frozen);
        let later = document
            .nodes
            .iter()
            .enumerate()
            .find_map(|(id, node)| {
                (id != first && matches!(&node.kind, NodeKind::Element(el) if el.tag == "base"))
                    .then_some(id)
            })
            .unwrap();
        document.set_attr(later, "href", "javascript:blocked");
        document
            .replace_text_content_owned(parent, "".into(), &mut budget())
            .unwrap();
        assert_eq!(document.frozen_base().unwrap().0, later);
        assert_eq!(
            document.base_url().as_str(),
            "https://example.test/changed/"
        );
    }

    #[test]
    fn template_contents_stay_separate_and_host_depth_is_checked() {
        let mut document =
            Document::parse("<template><details open name=n>inside</details></template>");
        let template = document.query_selector("template").unwrap();
        let content = document.template_contents(template).unwrap();
        let inside = document.nodes[content].children.clone();
        let groups = document.details_groups.clone();
        document
            .replace_text_content_owned(template, "outside".into(), &mut budget())
            .unwrap();
        assert_eq!(document.nodes[content].children, inside);
        assert_eq!(document.details_groups, groups);
        assert_eq!(document.text_content(content), "inside");
        let mut cursor = template;
        for _ in 0..MAX_DEPTH {
            let ancestor = document.create_element("div");
            document.nodes[cursor].parent = Some(ancestor);
            document.nodes[ancestor].children.push(cursor);
            cursor = ancestor;
        }
        let before = snapshot(&document);
        assert_eq!(
            document.replace_text_content_owned(content, "x".into(), &mut budget()),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(snapshot(&document), before);
    }

    #[test]
    fn no_hook_fast_path_ignores_unrelated_arena_and_budget_failures_are_not_refunded() {
        let (mut document, parent) = basic();
        let mut small = budget();
        document
            .replace_text_content_owned(parent, "".into(), &mut small)
            .unwrap();
        let (mut large, large_parent) = basic();
        for _ in 0..1000 {
            large.create_comment("unrelated");
        }
        let mut large_budget = budget();
        large
            .replace_text_content_owned(large_parent, "".into(), &mut large_budget)
            .unwrap();
        assert_eq!(
            (small.steps, small.allocated),
            (large_budget.steps, large_budget.allocated)
        );
        let mut meter = DomMutationBudget {
            steps: 4,
            allocated: 7,
            heap_limit: 8,
        };
        assert_eq!(meter.work(5), Err(DomDataError::LimitExceeded));
        assert_eq!(meter.steps, 0);
        assert_eq!(meter.charge(2), Err(DomDataError::LimitExceeded));
        assert_eq!(meter.allocated, 9);
        meter.allocated = usize::MAX;
        assert_eq!(meter.charge(1), Err(DomDataError::LimitExceeded));
        assert_eq!(meter.allocated, usize::MAX);
    }
}
