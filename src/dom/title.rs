//! Paid title selection, direct-Text normalization and narrow fresh insertion.
//! https://html.spec.whatwg.org/multipage/dom.html#document.title
use super::*;
use std::mem::size_of;

// Valid arenas retain at most B = MAX_DOM_BYTES and visit at most N nodes.
// Including empty Text entries, two normalization/projection passes plus
// output cost <= 29B + 10N + 34;
// selection costs < 24N plus bounded frame growth. This also stops repeated
// edges in malformed public Documents instead of trusting their byte ledger.
pub(super) const HOST_TITLE_WORK: usize = 40 * MAX_DOM_BYTES + 64 * MAX_NODES + 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TitleMode {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TitleTarget {
    None,
    Existing(NodeId),
    Create {
        parent: NodeId,
        namespace: Namespace,
        first: bool,
    },
}

fn add(a: usize, b: usize) -> Result<usize, DomDataError> {
    a.checked_add(b).ok_or(DomDataError::LimitExceeded)
}
fn mul(a: usize, b: usize) -> Result<usize, DomDataError> {
    a.checked_mul(b).ok_or(DomDataError::LimitExceeded)
}

fn named(
    doc: &Document,
    id: NodeId,
    namespace: Namespace,
    name: &str,
    budget: &mut DomMutationBudget,
) -> Result<bool, DomDataError> {
    budget.work(2)?;
    let node = doc.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
    let NodeKind::Element(element) = &node.kind else {
        return Ok(false);
    };
    if element.namespace != namespace || element.tag.len() != name.len() {
        return Ok(false);
    }
    budget.work(name.len())?;
    Ok(element.tag == name)
}

fn document_element(
    doc: &Document,
    budget: &mut DomMutationBudget,
) -> Result<Option<NodeId>, DomDataError> {
    budget.work(4)?;
    let root = doc.nodes.get(doc.root).ok_or(DomDataError::InvalidNode)?;
    if !matches!(root.kind, NodeKind::Document) || root.parent.is_some() {
        return Err(DomDataError::InvalidNode);
    }
    if root.children.len() > MAX_NODES {
        return Err(DomDataError::LimitExceeded);
    }
    for &id in &root.children {
        budget.work(4)?;
        let node = doc.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
        if node.parent != Some(doc.root) {
            return Err(DomDataError::InvalidNode);
        }
        if matches!(node.kind, NodeKind::Element(_)) {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

fn direct_named(
    doc: &Document,
    parent: NodeId,
    namespace: Namespace,
    name: &str,
    budget: &mut DomMutationBudget,
) -> Result<Option<NodeId>, DomDataError> {
    budget.work(2)?;
    let node = doc.nodes.get(parent).ok_or(DomDataError::InvalidNode)?;
    if node.children.len() > MAX_NODES {
        return Err(DomDataError::LimitExceeded);
    }
    for &id in &node.children {
        budget.work(4)?;
        if doc.nodes.get(id).ok_or(DomDataError::InvalidNode)?.parent != Some(parent) {
            return Err(DomDataError::InvalidNode);
        }
        if named(doc, id, namespace, name, budget)? {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

fn push_frame(
    frames: &mut Vec<(NodeId, usize)>,
    id: NodeId,
    budget: &mut DomMutationBudget,
) -> Result<(), DomDataError> {
    // Snapshot depth counts edges from Document at zero; the stack includes it.
    let maximum_frames = add(MAX_DEPTH, 1)?;
    if frames.len() >= maximum_frames {
        return Err(DomDataError::LimitExceeded);
    }
    if frames.len() == frames.capacity() {
        let capacity = add(frames.len(), 1)?.max(mul(frames.capacity(), 2)?.min(maximum_frames));
        budget.work(add(1, mul(frames.len(), 2)?)?)?;
        budget.charge(mul(capacity, size_of::<(NodeId, usize)>())?)?;
        frames
            .try_reserve_exact(capacity - frames.len())
            .map_err(|_| DomDataError::AllocationFailed)?;
    }
    budget.work(1)?;
    frames.push((id, 0));
    Ok(())
}

fn first_html_title(
    doc: &Document,
    budget: &mut DomMutationBudget,
) -> Result<Option<NodeId>, DomDataError> {
    let mut frames = Vec::new();
    push_frame(&mut frames, doc.root, budget)?;
    let mut visited = 1usize;
    // Root identity was checked by document_element. Only ordinary child
    // edges participate: template content and host associations are not edges.
    loop {
        budget.work(1)?;
        let depth = frames.len();
        let Some((parent, next)) = frames.last_mut() else {
            return Ok(None);
        };
        budget.work(2)?;
        let node = doc.nodes.get(*parent).ok_or(DomDataError::InvalidNode)?;
        if node.children.len() > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        let Some(&id) = node.children.get(*next) else {
            budget.work(1)?;
            frames.pop();
            continue;
        };
        *next += 1;
        budget.work(4)?;
        if depth > MAX_DEPTH {
            return Err(DomDataError::LimitExceeded);
        }
        visited = add(visited, 1)?;
        if visited > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        let child = doc.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
        if child.parent != Some(*parent) {
            return Err(DomDataError::InvalidNode);
        }
        if named(doc, id, Namespace::Html, "title", budget)? {
            return Ok(Some(id));
        }
        push_frame(&mut frames, id, budget)?;
    }
}

// A None budget is used only for a private second pass prepaid from the first
// pass over the same immutable document. Its actual work is still counted.
fn spend(
    budget: &mut Option<&mut DomMutationBudget>,
    work: &mut usize,
    amount: usize,
) -> Result<(), DomDataError> {
    *work = add(*work, amount)?;
    if let Some(budget) = budget.as_deref_mut() {
        budget.work(amount)?;
    }
    Ok(())
}

struct Normalized<'a> {
    doc: &'a Document,
    parent: NodeId,
    children: &'a [NodeId],
    next_child: usize,
    units: Option<DomUnits<'a>>,
    started: bool,
    pending_space: bool,
    deferred: Option<u16>,
    finished: bool,
    work: usize,
}

impl<'a> Normalized<'a> {
    fn new(
        doc: &'a Document,
        parent: NodeId,
        mut budget: Option<&mut DomMutationBudget>,
    ) -> Result<Self, DomDataError> {
        let mut work = 0;
        spend(&mut budget, &mut work, 8)?;
        let node = doc.nodes.get(parent).ok_or(DomDataError::InvalidNode)?;
        if !matches!(node.kind, NodeKind::Element(_)) {
            return Err(DomDataError::InvalidNode);
        }
        if node.children.len() > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        Ok(Self {
            doc,
            parent,
            children: &node.children,
            next_child: 0,
            units: None,
            started: false,
            pending_space: false,
            deferred: None,
            finished: false,
            work,
        })
    }

    fn raw(
        &mut self,
        budget: &mut Option<&mut DomMutationBudget>,
    ) -> Result<Option<u16>, DomDataError> {
        loop {
            if let Some(units) = &mut self.units {
                // Iterator advancement/EOF was prepaid from stored_bytes.
                if let Some(unit) = units.next() {
                    spend(budget, &mut self.work, 4)?;
                    return Ok(Some(unit));
                }
                self.units = None;
            }
            let Some(&id) = self.children.get(self.next_child) else {
                if !self.finished {
                    spend(budget, &mut self.work, 1)?;
                    self.finished = true;
                }
                return Ok(None);
            };
            spend(budget, &mut self.work, 4)?;
            self.next_child += 1;
            let child = self.doc.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
            if child.parent != Some(self.parent) {
                return Err(DomDataError::InvalidNode);
            }
            if let NodeKind::Text(text) = &child.kind {
                if !child.children.is_empty() {
                    return Err(DomDataError::InvalidNode);
                }
                spend(budget, &mut self.work, add(1, text.stored_bytes())?)?;
                self.units = Some(text.units());
            }
        }
    }

    fn next(
        &mut self,
        mut budget: Option<&mut DomMutationBudget>,
    ) -> Result<Option<u16>, DomDataError> {
        if let Some(unit) = self.deferred.take() {
            return Ok(Some(unit));
        }
        while let Some(unit) = self.raw(&mut budget)? {
            if matches!(unit, 9 | 10 | 12 | 13 | 32) {
                self.pending_space |= self.started;
                continue;
            }
            if self.pending_space {
                self.pending_space = false;
                self.deferred = Some(unit);
                return Ok(Some(32));
            }
            self.started = true;
            return Ok(Some(unit));
        }
        Ok(None)
    }
}

struct Projection<'a> {
    source: Normalized<'a>,
    pending: Option<u16>,
    decoder_work: usize,
}
impl Projection<'_> {
    fn work(&self) -> Result<usize, DomDataError> {
        add(self.source.work, self.decoder_work)
    }
    fn next(
        &mut self,
        mut budget: Option<&mut DomMutationBudget>,
    ) -> Result<Option<char>, DomDataError> {
        // Fixed decoder/state arithmetic, including a reached EOF attempt.
        spend(&mut budget, &mut self.decoder_work, 8)?;
        let first = match self.pending.take() {
            Some(unit) => Some(unit),
            None => self.source.next(budget.as_deref_mut())?,
        };
        let Some(first) = first else { return Ok(None) };
        if (0xd800..=0xdbff).contains(&first) {
            match self.source.next(budget)? {
                Some(second) if (0xdc00..=0xdfff).contains(&second) => {
                    let scalar =
                        0x10000 + ((u32::from(first) - 0xd800) << 10) + u32::from(second) - 0xdc00;
                    Ok(char::from_u32(scalar))
                }
                other => {
                    self.pending = other;
                    Ok(Some(char::REPLACEMENT_CHARACTER))
                }
            }
        } else if (0xdc00..=0xdfff).contains(&first) {
            Ok(Some(char::REPLACEMENT_CHARACTER))
        } else {
            Ok(char::from_u32(u32::from(first)))
        }
    }
}

impl Document {
    pub(crate) fn title_target(
        &self,
        mode: TitleMode,
        budget: &mut DomMutationBudget,
    ) -> Result<TitleTarget, DomDataError> {
        budget.work(8)?;
        let root = document_element(self, budget)?;
        if let Some(root) = root {
            if named(self, root, Namespace::Svg, "svg", budget)? {
                return Ok(
                    match direct_named(self, root, Namespace::Svg, "title", budget)? {
                        Some(id) => TitleTarget::Existing(id),
                        None if mode == TitleMode::Write => TitleTarget::Create {
                            parent: root,
                            namespace: Namespace::Svg,
                            first: true,
                        },
                        None => TitleTarget::None,
                    },
                );
            }
            budget.work(2)?;
            if mode == TitleMode::Write
                && !matches!(&self.nodes[root].kind, NodeKind::Element(e) if e.namespace == Namespace::Html)
            {
                return Ok(TitleTarget::None);
            }
        } else if mode == TitleMode::Write {
            return Ok(TitleTarget::None);
        }
        if let Some(id) = first_html_title(self, budget)? {
            return Ok(TitleTarget::Existing(id));
        }
        if mode == TitleMode::Write
            && let Some(root) = root
            && named(self, root, Namespace::Html, "html", budget)?
            && let Some(head) = direct_named(self, root, Namespace::Html, "head", budget)?
        {
            return Ok(TitleTarget::Create {
                parent: head,
                namespace: Namespace::Html,
                first: false,
            });
        }
        Ok(TitleTarget::None)
    }

    pub(crate) fn title_units_bounded(
        &self,
        maximum_units: usize,
        budget: &mut DomMutationBudget,
    ) -> Result<Vec<u16>, DomDataError> {
        let TitleTarget::Existing(id) = self.title_target(TitleMode::Read, budget)? else {
            budget.charge(32)?;
            return Ok(Vec::new());
        };
        let mut plan = Normalized::new(self, id, Some(&mut *budget))?;
        let mut count = 0;
        while plan.next(Some(&mut *budget))?.is_some() {
            count = add(count, 1)?;
            if count > maximum_units {
                return Err(DomDataError::LimitExceeded);
            }
        }
        budget.work(add(plan.work, count)?)?;
        budget.charge(add(32, mul(count, size_of::<u16>())?)?)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(count)
            .map_err(|_| DomDataError::AllocationFailed)?;
        let mut emit = Normalized::new(self, id, None)?;
        while let Some(unit) = emit.next(None)? {
            if output.len() == count {
                return Err(DomDataError::InvalidData);
            }
            output.push(unit);
        }
        if output.len() != count || emit.work != plan.work {
            return Err(DomDataError::InvalidData);
        }
        Ok(output)
    }

    pub(crate) fn title_projection_bounded(
        &self,
        maximum_bytes: usize,
        budget: &mut DomMutationBudget,
    ) -> Result<String, DomDataError> {
        let TitleTarget::Existing(id) = self.title_target(TitleMode::Read, budget)? else {
            budget.charge(32)?;
            return Ok(String::new());
        };
        let mut plan = Projection {
            source: Normalized::new(self, id, Some(&mut *budget))?,
            pending: None,
            decoder_work: 0,
        };
        let mut bytes = 0;
        let mut count = 0;
        while let Some(value) = plan.next(Some(&mut *budget))? {
            let next = add(bytes, value.len_utf8())?;
            if next > maximum_bytes.min(MAX_TEXT) {
                break;
            }
            bytes = next;
            count = add(count, 1)?;
        }
        budget.work(add(plan.work()?, bytes)?)?;
        budget.charge(add(32, bytes)?)?;
        let mut output = String::new();
        output
            .try_reserve_exact(bytes)
            .map_err(|_| DomDataError::AllocationFailed)?;
        let mut emit = Projection {
            source: Normalized::new(self, id, None)?,
            pending: None,
            decoder_work: 0,
        };
        for _ in 0..count {
            let value = emit.next(None)?.ok_or(DomDataError::InvalidData)?;
            if add(output.len(), value.len_utf8())? > bytes {
                return Err(DomDataError::InvalidData);
            }
            output.push(value);
        }
        if output.len() != bytes || emit.work()? > plan.work()? {
            return Err(DomDataError::InvalidData);
        }
        Ok(output)
    }

    pub(crate) fn create_title_element(
        &mut self,
        namespace: Namespace,
        budget: &mut DomMutationBudget,
    ) -> Result<NodeId, DomDataError> {
        budget.work(8)?;
        if !matches!(namespace, Namespace::Html | Namespace::Svg) {
            return Err(DomDataError::InvalidData);
        }
        let retained = add(self.retained_bytes, 5)?;
        if self.nodes.len() >= MAX_NODES || retained > MAX_DOM_BYTES {
            return Err(DomDataError::LimitExceeded);
        }
        budget.work(5 + 8)?;
        budget.charge(128 + 5)?;
        let mut tag = String::new();
        tag.try_reserve_exact(5)
            .map_err(|_| DomDataError::AllocationFailed)?;
        tag.push_str("title");
        if self.nodes.len() == self.nodes.capacity() {
            budget.work(add(1, mul(self.nodes.len(), 2)?)?)?;
            budget.charge(mul(add(self.nodes.len(), 1)?, size_of::<Node>())?)?;
            self.nodes
                .try_reserve_exact(1)
                .map_err(|_| DomDataError::AllocationFailed)?;
        }
        // All fallible work precedes this detached-node publication.
        let id = self.nodes.len();
        self.nodes.push(Node {
            parent: None,
            children: Vec::new(),
            kind: NodeKind::Element(Element {
                namespace,
                tag,
                attrs: BTreeMap::new(),
                attr_namespaces: BTreeMap::new(),
                template_contents: None,
            }),
        });
        self.retained_bytes = retained;
        Ok(id)
    }

    pub(crate) fn insert_title_element(
        &mut self,
        parent: NodeId,
        title: NodeId,
        first: bool,
        budget: &mut DomMutationBudget,
    ) -> Result<(), DomDataError> {
        budget.work(8)?;
        let root = document_element(self, budget)?.ok_or(DomDataError::InvalidNode)?;
        let namespace = if first {
            if root != parent || !named(self, root, Namespace::Svg, "svg", budget)? {
                return Err(DomDataError::InvalidNode);
            }
            Namespace::Svg
        } else {
            if !named(self, root, Namespace::Html, "html", budget)?
                || direct_named(self, root, Namespace::Html, "head", budget)? != Some(parent)
            {
                return Err(DomDataError::InvalidNode);
            }
            Namespace::Html
        };
        if !named(self, title, namespace, "title", budget)? {
            return Err(DomDataError::InvalidNode);
        }
        let node = self.nodes.get(title).ok_or(DomDataError::InvalidNode)?;
        let NodeKind::Element(element) = &node.kind else {
            return Err(DomDataError::InvalidNode);
        };
        if node.parent.is_some()
            || !node.children.is_empty()
            || !element.attrs.is_empty()
            || !element.attr_namespaces.is_empty()
            || element.template_contents.is_some()
            || title == parent
        {
            return Err(DomDataError::InvalidNode);
        }
        let mut cursor = Some(parent);
        let mut ancestors = 0;
        while let Some(id) = cursor {
            budget.work(4)?;
            ancestors += 1;
            if ancestors >= MAX_DEPTH {
                return Err(DomDataError::LimitExceeded);
            }
            let node = self.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
            if id == title || (node.parent.is_none() && id != self.root) {
                return Err(DomDataError::InvalidNode);
            }
            cursor = node.parent;
        }
        let children = &self.nodes[parent].children;
        if children.len() >= MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        for &id in children {
            budget.work(4)?;
            if self.nodes.get(id).ok_or(DomDataError::InvalidNode)?.parent != Some(parent) {
                return Err(DomDataError::InvalidNode);
            }
        }
        budget.work(add(8, if first { children.len() } else { 0 })?)?;
        if children.len() == children.capacity() {
            budget.work(add(1, mul(children.len(), 2)?)?)?;
            budget.charge(mul(add(children.len(), 1)?, size_of::<NodeId>())?)?;
            self.nodes[parent]
                .children
                .try_reserve_exact(1)
                .map_err(|_| DomDataError::AllocationFailed)?;
        }
        // This narrow insertion adds only an empty title beneath the actual
        // head/SVG root. It cannot move details or affect base selection, so no
        // general raw insertion or its infallible BTree/URL machinery is used.
        if first {
            self.nodes[parent].children.insert(0, title);
        } else {
            self.nodes[parent].children.push(title);
        }
        self.nodes[title].parent = Some(parent);
        Ok(())
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

    fn exact(units: &[u16]) -> DomString {
        DomString::from_units_owned(units.to_vec()).unwrap()
    }

    fn root_document(namespace: Namespace, tag: &str) -> (Document, NodeId) {
        let mut doc = Document::parse("");
        doc.clear_children(doc.root);
        let root = doc.create_element_ns(namespace, tag);
        doc.append_child(doc.root, root);
        (doc, root)
    }

    fn text(doc: &mut Document, parent: NodeId, units: &[u16]) -> NodeId {
        let id = doc.create_text_node_owned(exact(units)).unwrap();
        doc.append_child(parent, id);
        id
    }

    #[test]
    fn direct_text_normalization_preserves_units_and_ignores_other_children() {
        let mut doc = Document::parse("<title></title>");
        let title = doc.query_selector("title").unwrap();
        let first = text(&mut doc, title, &[9, 32, 65, 0xd83d]);
        let comment = doc.create_comment("ignored");
        doc.append_child(title, comment);
        let nested = doc.create_element("span");
        doc.append_child(title, nested);
        text(&mut doc, nested, &[88]);
        text(&mut doc, title, &[0xde80, 13, 10]);
        text(&mut doc, title, &[12, 32, 66, 160, 11, 0xd800, 32]);
        let before = format!("{doc:?}");
        assert_eq!(
            doc.title_units_bounded(8, &mut budget()).unwrap(),
            [65, 0xd83d, 0xde80, 32, 66, 160, 11, 0xd800]
        );
        assert_eq!(doc.title(), "A🚀 B\u{a0}\u{b}�");
        assert_eq!(format!("{doc:?}"), before);
        assert!(matches!(&doc.nodes[first].kind, NodeKind::Text(data)
            if data.units().collect::<Vec<_>>() == [9, 32, 65, 0xd83d]));
    }

    #[test]
    fn selection_respects_actual_root_namespace_direct_svg_and_html_tree_order() {
        let (mut doc, root) = root_document(Namespace::Svg, "svg");
        let nested = doc.create_element_ns(Namespace::Svg, "g");
        doc.append_child(root, nested);
        let skipped = doc.create_element_ns(Namespace::Svg, "title");
        doc.append_child(nested, skipped);
        let html = doc.create_element("title");
        doc.append_child(root, html);
        assert_eq!(
            doc.title_target(TitleMode::Read, &mut budget()),
            Ok(TitleTarget::None)
        );
        assert_eq!(
            doc.title_target(TitleMode::Write, &mut budget()),
            Ok(TitleTarget::Create {
                parent: root,
                namespace: Namespace::Svg,
                first: true,
            })
        );
        let direct = doc.create_element_ns(Namespace::Svg, "title");
        doc.append_child(root, direct);
        assert_eq!(
            doc.title_target(TitleMode::Read, &mut budget()),
            Ok(TitleTarget::Existing(direct))
        );

        let (mut doc, root) = root_document(Namespace::MathMl, "math");
        let title = doc.create_element("title");
        doc.append_child(root, title);
        assert_eq!(
            doc.title_target(TitleMode::Read, &mut budget()),
            Ok(TitleTarget::Existing(title))
        );
        assert_eq!(
            doc.title_target(TitleMode::Write, &mut budget()),
            Ok(TitleTarget::None)
        );
        let (mut doc, root) = root_document(Namespace::Html, "div");
        assert_eq!(
            doc.title_target(TitleMode::Write, &mut budget()),
            Ok(TitleTarget::None)
        );
        let title = doc.create_element("title");
        doc.append_child(root, title);
        assert_eq!(
            doc.title_target(TitleMode::Write, &mut budget()),
            Ok(TitleTarget::Existing(title))
        );
    }

    #[test]
    fn template_contents_are_excluded_but_ordinary_template_children_are_reached() {
        let mut doc = Document::parse("<template><title>inert</title></template>");
        let template = doc.query_selector("template").unwrap();
        assert_eq!(
            doc.title_target(TitleMode::Read, &mut budget()),
            Ok(TitleTarget::None)
        );
        let title = doc.create_element("title");
        doc.append_child(template, title);
        text(&mut doc, title, &[65]);
        assert_eq!(
            doc.title_target(TitleMode::Read, &mut budget()),
            Ok(TitleTarget::Existing(title))
        );
        assert_eq!(doc.title(), "A");
        // A title found earlier in tree order does not inspect a later branch.
        let body = doc.query_selector("body").unwrap();
        doc.nodes[body].children.push(usize::MAX);
        assert_eq!(
            doc.title_target(TitleMode::Read, &mut budget()),
            Ok(TitleTarget::Existing(title))
        );
    }

    #[test]
    fn getter_limits_apply_to_normalized_output_and_refusals_do_not_mutate() {
        let doc = Document::parse("<title> \t\n A \r\n </title>");
        let before = format!("{doc:?}");
        assert_eq!(doc.title_units_bounded(1, &mut budget()).unwrap(), [65]);
        assert_eq!(
            doc.title_units_bounded(0, &mut budget()),
            Err(DomDataError::LimitExceeded)
        );
        let blank = Document::parse("<title> \t\n\r\u{c} </title>");
        assert!(
            blank
                .title_units_bounded(0, &mut budget())
                .unwrap()
                .is_empty()
        );
        for projection in [false, true] {
            let run = |meter: &mut DomMutationBudget| {
                if projection {
                    doc.title_projection_bounded(8, meter).map(|s| s.len())
                } else {
                    doc.title_units_bounded(8, meter).map(|s| s.len())
                }
            };
            let mut measured = budget();
            measured.allocated = 17;
            assert_eq!(run(&mut measured), Ok(1));
            let work = 100_000 - measured.steps;
            let mut exact = DomMutationBudget {
                steps: work,
                allocated: 17,
                heap_limit: measured.allocated,
            };
            assert_eq!(run(&mut exact), Ok(1));
            assert_eq!(exact.steps, 0);
            let mut short = DomMutationBudget {
                steps: work - 1,
                allocated: 17,
                heap_limit: measured.allocated,
            };
            assert_eq!(run(&mut short), Err(DomDataError::LimitExceeded));
            assert_eq!(short.steps, 0);
            let mut short = DomMutationBudget {
                steps: work,
                allocated: 17,
                heap_limit: measured.allocated - 1,
            };
            assert_eq!(run(&mut short), Err(DomDataError::LimitExceeded));
            assert_eq!(short.allocated, measured.allocated);
        }
        assert_eq!(format!("{doc:?}"), before);
    }

    #[test]
    fn host_projection_keeps_a_scalar_prefix_without_resuming_after_overflow() {
        let mut doc = Document::parse("<title></title>");
        let title = doc.query_selector("title").unwrap();
        text(&mut doc, title, &[65, 0x20ac, 66, 0xd800]);
        assert_eq!(doc.title_projection_bounded(3, &mut budget()).unwrap(), "A");
        assert_eq!(
            doc.title_projection_bounded(4, &mut budget()).unwrap(),
            "A€"
        );
        assert_eq!(
            doc.title_projection_bounded(7, &mut budget()).unwrap(),
            "A€B"
        );
        assert_eq!(
            doc.title_projection_bounded(8, &mut budget()).unwrap(),
            "A€B�"
        );
        doc.clear_children(title);
        let first = doc.create_text_node(&"a".repeat(MAX_TEXT - 1));
        doc.append_child(title, first);
        let suffix = doc.create_text_node("€z");
        doc.append_child(title, suffix);
        let projected = doc.title();
        assert_eq!(projected.len(), MAX_TEXT - 1);
        assert!(projected.bytes().all(|b| b == b'a'));
    }

    #[test]
    fn repeated_whitespace_edges_consume_work_before_unbounded_rescanning() {
        let mut doc = Document::parse("<title></title>");
        let title = doc.query_selector("title").unwrap();
        let id = doc.create_text_node(&" ".repeat(4096));
        doc.append_child(title, id);
        // Public fields can form this malformed duplicate-edge graph, unlike
        // checked mutation and snapshot decoding. Each rescan must debit work.
        doc.nodes[title].children = vec![id; 128];
        let before = format!("{doc:?}");
        let mut meter = budget();
        assert_eq!(
            doc.title_projection_bounded(MAX_TEXT, &mut meter),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(meter.steps, 0);
        assert!(meter.allocated < 1024); // No output is allocated before planning.
        assert_eq!(format!("{doc:?}"), before);
    }

    #[test]
    fn accepted_snapshot_depth_limit_is_not_shortened_by_document_frame() {
        let (mut doc, mut parent) = root_document(Namespace::Html, "html");
        for depth in 2..=MAX_DEPTH {
            let child = doc.create_element(if depth == MAX_DEPTH { "title" } else { "div" });
            // Set up the snapshot directly: raw append has a separate guard.
            doc.nodes[parent].children.push(child);
            doc.nodes[child].parent = Some(parent);
            parent = child;
        }
        let doc =
            Document::from_snapshot(doc.nodes.clone(), doc.root, false, DocumentMode::NoQuirks)
                .unwrap();
        assert_eq!(
            doc.title_target(TitleMode::Read, &mut budget()),
            Ok(TitleTarget::Existing(parent))
        );
        let mut too_deep = doc.clone();
        if let NodeKind::Element(element) = &mut too_deep.nodes[parent].kind {
            element.tag = "div".into();
        }
        let child = too_deep.create_element("title");
        too_deep.nodes[parent].children.push(child);
        too_deep.nodes[child].parent = Some(parent);
        assert_eq!(
            too_deep.title_target(TitleMode::Read, &mut budget()),
            Err(DomDataError::LimitExceeded)
        );
    }

    fn creation_case(growth: bool) -> Document {
        let mut doc = Document::parse("");
        doc.nodes = doc.nodes.into_boxed_slice().into_vec();
        if !growth {
            doc.nodes.reserve_exact(1);
        }
        assert_eq!(doc.nodes.len() == doc.nodes.capacity(), growth);
        doc
    }

    #[test]
    fn fresh_title_creation_exact_and_one_short_admissions_preserve_outer_state() {
        for namespace in [Namespace::Html, Namespace::Svg] {
            for growth in [false, true] {
                let mut measured_doc = creation_case(growth);
                let mut measured = budget();
                let id = measured_doc
                    .create_title_element(namespace, &mut measured)
                    .unwrap();
                assert_eq!(measured_doc.nodes[id].parent, None);
                assert!(measured_doc.nodes[id].children.is_empty());
                assert!(matches!(&measured_doc.nodes[id].kind, NodeKind::Element(e)
                    if e.namespace == namespace && e.tag == "title" && e.attrs.is_empty()
                        && e.attr_namespaces.is_empty() && e.template_contents.is_none()));
                let work = 100_000 - measured.steps;
                for (steps, heap, succeeds) in [
                    (work, measured.allocated, true),
                    (work - 1, measured.allocated, false),
                    (work, measured.allocated - 1, false),
                ] {
                    let mut doc = creation_case(growth);
                    let before = format!("{doc:?}");
                    let capacity = doc.nodes.capacity();
                    let old_len = doc.nodes.len();
                    let retained = doc.retained_bytes;
                    let mut meter = DomMutationBudget {
                        steps,
                        allocated: 0,
                        heap_limit: heap,
                    };
                    let result = doc.create_title_element(namespace, &mut meter);
                    if succeeds {
                        assert_eq!(result, Ok(old_len));
                        assert_eq!(doc.retained_bytes, retained + 5);
                        assert_eq!(meter.steps, 0);
                    } else {
                        assert_eq!(result, Err(DomDataError::LimitExceeded));
                        assert_eq!(format!("{doc:?}"), before);
                        assert_eq!(doc.nodes.capacity(), capacity);
                    }
                }
            }
        }
    }

    fn insertion_case(svg: bool, growth: bool) -> (Document, NodeId, NodeId, NodeId) {
        let (mut doc, parent) = if svg {
            root_document(Namespace::Svg, "svg")
        } else {
            let doc = Document::parse("");
            let head = doc.query_selector("head").unwrap();
            (doc, head)
        };
        let old = doc.create_comment("kept");
        doc.append_child(parent, old);
        let title = doc
            .create_title_element(
                if svg { Namespace::Svg } else { Namespace::Html },
                &mut budget(),
            )
            .unwrap();
        let children = std::mem::take(&mut doc.nodes[parent].children);
        doc.nodes[parent].children = children.into_boxed_slice().into_vec();
        if !growth {
            doc.nodes[parent].children.reserve_exact(1);
        }
        assert_eq!(
            doc.nodes[parent].children.len() == doc.nodes[parent].children.capacity(),
            growth
        );
        (doc, parent, title, old)
    }

    #[test]
    fn insertion_exact_and_one_short_keep_created_title_detached_on_refusal() {
        for svg in [false, true] {
            for growth in [false, true] {
                let (mut doc, parent, title, _) = insertion_case(svg, growth);
                let mut measured = budget();
                doc.insert_title_element(parent, title, svg, &mut measured)
                    .unwrap();
                let work = 100_000 - measured.steps;
                let mut limits = vec![
                    (work, measured.allocated, true),
                    (work - 1, measured.allocated, false),
                ];
                if measured.allocated > 0 {
                    limits.push((work, measured.allocated - 1, false));
                }
                for (steps, heap, succeeds) in limits {
                    let (mut doc, parent, title, old) = insertion_case(svg, growth);
                    let before = format!("{doc:?}");
                    let capacity = doc.nodes[parent].children.capacity();
                    let retained = doc.retained_bytes;
                    let mut meter = DomMutationBudget {
                        steps,
                        allocated: 0,
                        heap_limit: heap,
                    };
                    let result = doc.insert_title_element(parent, title, svg, &mut meter);
                    if succeeds {
                        assert_eq!(result, Ok(()));
                        assert_eq!(
                            doc.nodes[parent].children,
                            if svg {
                                vec![title, old]
                            } else {
                                vec![old, title]
                            }
                        );
                        assert_eq!(doc.nodes[title].parent, Some(parent));
                        assert_eq!(doc.retained_bytes, retained);
                        assert_eq!(meter.steps, 0);
                    } else {
                        assert_eq!(result, Err(DomDataError::LimitExceeded));
                        assert_eq!(doc.nodes[title].parent, None);
                        assert_eq!(format!("{doc:?}"), before);
                        assert_eq!(doc.nodes[parent].children.capacity(), capacity);
                    }
                }
            }
        }
    }

    #[test]
    fn insertion_rejects_invalid_identity_links_and_nonempty_fresh_title() {
        for variant in 0..5 {
            let (mut doc, mut parent, mut title, old) = insertion_case(false, true);
            match variant {
                0 => parent = usize::MAX,
                1 => title = usize::MAX,
                2 => doc.nodes[old].parent = None,
                3 => {
                    doc.set_attr(title, "id", "not-fresh");
                }
                4 => {
                    if let NodeKind::Element(e) = &mut doc.nodes[title].kind {
                        e.namespace = Namespace::Svg;
                    }
                }
                _ => unreachable!(),
            }
            let before = format!("{doc:?}");
            assert_eq!(
                doc.insert_title_element(parent, title, false, &mut budget()),
                Err(DomDataError::InvalidNode)
            );
            assert_eq!(format!("{doc:?}"), before);
        }
    }

    #[test]
    fn final_node_slot_preserves_inserted_empty_title_before_content_refusal() {
        let mut doc = Document::parse("");
        let head = doc.query_selector("head").unwrap();
        while doc.nodes.len() < MAX_NODES - 1 {
            doc.nodes.push(Node {
                parent: None,
                children: Vec::new(),
                kind: NodeKind::Comment("".into()),
            });
        }
        // Test-owned spare capacity isolates the last logical slot from growth work.
        doc.nodes.reserve_exact(1);
        let title = doc
            .create_title_element(Namespace::Html, &mut budget())
            .unwrap();
        doc.insert_title_element(head, title, false, &mut budget())
            .unwrap();
        assert_eq!(doc.nodes.len(), MAX_NODES);
        assert_eq!(
            doc.check_text_content_replacement(title, 1),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(
            doc.replace_text_content_owned(title, "".into(), &mut budget()),
            Ok(None)
        );
        assert_eq!(doc.nodes[title].parent, Some(head));
        assert!(doc.nodes[title].children.is_empty());
        assert_eq!(
            doc.create_title_element(Namespace::Html, &mut budget()),
            Err(DomDataError::LimitExceeded)
        );
        let mut bytes = Document::parse("");
        bytes.retained_bytes = MAX_DOM_BYTES - 4;
        let before = format!("{bytes:?}");
        assert_eq!(
            bytes.create_title_element(Namespace::Svg, &mut budget()),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(format!("{bytes:?}"), before);
        assert_eq!(
            bytes.create_title_element(Namespace::MathMl, &mut budget()),
            Err(DomDataError::InvalidData)
        );
    }
}
