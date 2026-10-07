//! A bounded, independent HTML tree builder and CSS selector matcher.
use std::collections::{BTreeMap, BTreeSet};

mod mutation_budget;
mod node_clone;
mod node_normalize;
mod string;
#[cfg(test)]
mod string_tests;
mod text_operations;
mod text_replacement;
mod title;
pub(crate) use mutation_budget::DomMutationBudget;
pub use string::{DomScalars, DomString, DomUnits};
pub(crate) use title::{TitleMode, TitleTarget};

pub type NodeId = usize;
pub const MAX_NODES: usize = 100_000;
pub const MAX_DEPTH: usize = 256;
const MAX_TEXT: usize = 8 * 1024 * 1024;
pub(crate) const MAX_INLINE_STYLES: usize = 256;
pub(crate) const MAX_STYLE_CHILD_VISITS: usize = MAX_NODES * 2;
pub const MAX_DOM_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DocumentMode {
    #[default]
    NoQuirks,
    LimitedQuirks,
    Quirks,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Namespace {
    #[default]
    Html,
    Svg,
    MathMl,
}
impl Namespace {
    pub fn uri(self) -> &'static str {
        match self {
            Self::Html => "http://www.w3.org/1999/xhtml",
            Self::Svg => "http://www.w3.org/2000/svg",
            Self::MathMl => "http://www.w3.org/1998/Math/MathML",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeNamespace {
    XLink,
    Xml,
    Xmlns,
}
impl AttributeNamespace {
    pub fn uri(self) -> &'static str {
        match self {
            Self::XLink => "http://www.w3.org/1999/xlink",
            Self::Xml => "http://www.w3.org/XML/1998/namespace",
            Self::Xmlns => "http://www.w3.org/2000/xmlns/",
        }
    }
    /// Namespace adjustments defined by the HTML foreign-content parser.
    pub fn from_qualified_name(name: &str) -> Option<Self> {
        match name {
            "xlink:actuate" | "xlink:arcrole" | "xlink:href" | "xlink:role" | "xlink:show"
            | "xlink:title" | "xlink:type" => Some(Self::XLink),
            "xml:lang" | "xml:space" => Some(Self::Xml),
            "xmlns" | "xmlns:xlink" => Some(Self::Xmlns),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Element {
    pub namespace: Namespace,
    pub tag: String,
    pub attrs: BTreeMap<String, String>,
    pub attr_namespaces: BTreeMap<String, AttributeNamespace>,
    /// A separate inert fragment, never an ordinary child of this element.
    pub template_contents: Option<NodeId>,
}
impl Element {
    fn retained_bytes(&self) -> usize {
        self.tag.len()
            + self
                .attrs
                .iter()
                .map(|(name, value)| name.len() + value.len())
                .sum::<usize>()
            + self.attr_namespaces.keys().map(String::len).sum::<usize>()
    }
}
#[derive(Debug, Clone)]
pub struct Doctype {
    pub name: String,
    pub public_id: Option<String>,
    pub system_id: Option<String>,
    pub force_quirks: bool,
}
#[derive(Debug, Clone)]
pub enum NodeKind {
    Document,
    DocumentFragment { host: Option<NodeId> },
    Element(Element),
    Text(DomString),
    Comment(DomString),
    Doctype(Doctype),
    ProcessingInstruction { target: String, data: DomString },
}
/// Checked script-facing character-data storage refuses instead of truncating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomDataError {
    InvalidNode,
    InvalidData,
    LimitExceeded,
    AllocationFailed,
}

/// Allocation-free common text-content shapes shared with script admission.
pub(crate) enum TextContentShape<'a> {
    Empty,
    Single(&'a DomString),
    Tree,
}

enum TextSegments<'a> {
    Empty,
    One(&'a DomString),
    Many(Vec<&'a DomString>),
}
impl TextSegments<'_> {
    fn as_slice(&self) -> &[&DomString] {
        match self {
            Self::Empty => &[],
            Self::One(text) => std::slice::from_ref(text),
            Self::Many(texts) => texts,
        }
    }
    fn scalar_output(
        &self,
        maximum: usize,
        replace: bool,
        truncate: bool,
    ) -> Result<String, DomDataError> {
        if let Self::Empty = self {
            return Ok(String::new());
        }
        if let Self::One(text) = self
            && let Some(text) = text.scalar()
        {
            let length = if text.len() <= maximum {
                text.len()
            } else if truncate {
                floor_boundary(text, maximum)
            } else {
                return Err(DomDataError::LimitExceeded);
            };
            let mut output = String::new();
            output
                .try_reserve_exact(length)
                .map_err(|_| DomDataError::AllocationFailed)?;
            output.push_str(&text[..length]);
            return Ok(output);
        }
        string::scalar_from_units(
            self.as_slice().iter().flat_map(|text| text.units()),
            maximum,
            replace,
            truncate,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub kind: NodeKind,
}
#[derive(Debug, Clone)]
pub struct Document {
    pub nodes: Vec<Node>,
    pub root: NodeId,
    retained_bytes: usize,
    scripting_enabled: bool,
    mode: DocumentMode,
    encoding: &'static encoding_rs::Encoding,
    encoding_declaration: Option<&'static encoding_rs::Encoding>,
    url: url::Url,
    first_base: Option<(NodeId, url::Url)>,
    base_tracking: bool,
    base_href_bytes: usize,
    details_groups: BTreeMap<(NodeId, String), NodeId>,
    details_toggles: BTreeMap<u64, DetailsToggle>,
    details_trackers: BTreeMap<NodeId, DetailsToggleTracker>,
    details_active: Option<DetailsToggleTask>,
    details_sequence: u64,
    details_name_bytes: usize,
    details_summaries: std::cell::RefCell<BTreeMap<NodeId, Option<NodeId>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DetailsToggle {
    pub node: NodeId,
    pub old_open: bool,
    pub new_open: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DetailsToggleTask {
    pub id: u64,
    pub event: DetailsToggle,
}
#[derive(Debug, Clone, Copy)]
struct DetailsToggleTracker {
    task: u64,
    old_open: bool,
}

impl Default for Document {
    fn default() -> Self {
        Self::parse("")
    }
}
impl Document {
    /// Rebuild an arena received across the page-process boundary. Private byte
    /// accounting is recomputed; detached subtrees are checked as well.
    pub(crate) fn from_snapshot(
        nodes: Vec<Node>,
        root: NodeId,
        scripting_enabled: bool,
        mode: DocumentMode,
    ) -> Result<Self, String> {
        if nodes.is_empty()
            || nodes.len() > MAX_NODES
            || root >= nodes.len()
            || !matches!(nodes[root].kind, NodeKind::Document)
            || nodes[root].parent.is_some()
        {
            return Err("invalid snapshot document root".into());
        }
        let mut bytes = 0usize;
        let mut incoming = vec![false; nodes.len()];
        for (id, node) in nodes.iter().enumerate() {
            let own = match &node.kind {
                NodeKind::Document => {
                    if id != root {
                        return Err("duplicate document root".into());
                    }
                    0
                }
                NodeKind::DocumentFragment { host } => {
                    if node.parent.is_some() || host.is_some_and(|host| {
                        !matches!(nodes.get(host).map(|node| &node.kind), Some(NodeKind::Element(el)) if el.namespace == Namespace::Html && el.tag == "template" && el.template_contents == Some(id))
                    }) {
                        return Err("invalid snapshot fragment host".into());
                    }
                    0
                }
                NodeKind::Element(el) => {
                    if el.namespace == Namespace::Html && el.tag == "template" {
                        if !el.template_contents.is_some_and(|content| {
                            matches!(nodes.get(content).map(|node| &node.kind), Some(NodeKind::DocumentFragment { host: Some(host) }) if *host == id)
                        }) {
                            return Err("invalid snapshot template contents".into());
                        }
                    } else if el.template_contents.is_some() {
                        return Err("snapshot non-template has template contents".into());
                    }
                    if el.attrs.len() > 1024 {
                        return Err("snapshot element attribute limit exceeded".into());
                    }
                    if el.attr_namespaces.iter().any(|(name, namespace)| {
                        !el.attrs.contains_key(name)
                            || AttributeNamespace::from_qualified_name(name) != Some(*namespace)
                    }) {
                        return Err("invalid snapshot attribute namespace".into());
                    }
                    el.retained_bytes()
                }
                NodeKind::Text(s) | NodeKind::Comment(s) => s.stored_bytes(),
                NodeKind::Doctype(d) => {
                    d.name.len()
                        + d.public_id.as_ref().map_or(0, String::len)
                        + d.system_id.as_ref().map_or(0, String::len)
                }
                NodeKind::ProcessingInstruction { target, data } => {
                    target.len() + data.stored_bytes()
                }
            };
            bytes = bytes.checked_add(own).ok_or("snapshot byte overflow")?;
            if bytes > MAX_DOM_BYTES {
                return Err("snapshot DOM byte budget exceeded".into());
            }
            if !node.children.is_empty()
                && !matches!(
                    node.kind,
                    NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_)
                )
            {
                return Err("snapshot leaf has children".into());
            }
            for &child in &node.children {
                if child >= nodes.len()
                    || child == root
                    || matches!(nodes[child].kind, NodeKind::DocumentFragment { .. })
                    || incoming[child]
                    || nodes[child].parent != Some(id)
                    || matches!(nodes[child].kind, NodeKind::Doctype(_)) && id != root
                {
                    return Err("inconsistent snapshot parent/child links".into());
                }
                incoming[child] = true;
            }
        }
        if nodes
            .iter()
            .enumerate()
            .any(|(id, n)| n.parent.is_some() != incoming[id])
        {
            return Err("snapshot parent has no child link".into());
        }
        let mut visited = 0usize;
        let mut pending: Vec<_> = nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                n.parent.is_none()
                    && !matches!(n.kind, NodeKind::DocumentFragment { host: Some(_) })
            })
            .map(|(id, _)| (id, 0))
            .collect();
        while let Some((id, depth)) = pending.pop() {
            if depth > MAX_DEPTH {
                return Err("snapshot DOM depth exceeded".into());
            }
            visited += 1;
            if visited > nodes.len() {
                return Err("cyclic snapshot template contents".into());
            }
            pending.extend(nodes[id].children.iter().map(|&child| (child, depth + 1)));
            if let NodeKind::Element(el) = &nodes[id].kind
                && let Some(content) = el.template_contents
            {
                pending.push((content, depth + 1));
            }
        }
        if visited != nodes.len() {
            return Err("cyclic snapshot DOM".into());
        }
        let mut document = Self {
            nodes,
            root,
            retained_bytes: bytes,
            scripting_enabled,
            mode,
            encoding: encoding_rs::UTF_8,
            encoding_declaration: None,
            url: url::Url::parse("about:blank").expect("constant URL"),
            first_base: None,
            base_tracking: true,
            base_href_bytes: 0,
            details_groups: BTreeMap::new(),
            details_toggles: BTreeMap::new(),
            details_trackers: BTreeMap::new(),
            details_active: None,
            details_sequence: 0,
            details_name_bytes: 0,
            details_summaries: std::cell::RefCell::new(BTreeMap::new()),
        };
        for id in 0..document.nodes.len() {
            if document.html_control_tag(id) == Some("details") {
                document.details_name_bytes += document.attr(id, "name").map_or(0, str::len);
                if let Some(key) = document.details_group_key(id) {
                    document.details_groups.entry(key).or_insert(id);
                }
            }
        }
        Ok(document)
    }
    pub fn parse(source: &str) -> Self {
        parse(source)
    }
    pub fn parse_with_scripting(source: &str, scripting: bool) -> Self {
        parse_with_scripting(source, scripting)
    }
    /// Parse against an existing element without modifying its document. The
    /// returned arena's root directly contains the fragment children. Scripts
    /// are never executed. The context's document mode is inherited; template
    /// contents are separate fragment trees in the same bounded arena.
    pub fn parse_fragment(&self, context: NodeId, source: &str) -> Result<Self, String> {
        let context = match self.nodes.get(context).map(|node| &node.kind) {
            Some(NodeKind::DocumentFragment { host: Some(host) }) => *host,
            _ => context,
        };
        let Some(Node {
            kind: NodeKind::Element(element),
            ..
        }) = self.nodes.get(context)
        else {
            return Err("fragment context must be an element".into());
        };
        if element.retained_bytes() > MAX_DOM_BYTES / 2 || element.attrs.len() > 1024 {
            return Err("fragment context exceeds resource limit".into());
        }
        let mut form_ancestor = false;
        let mut mode = self.mode;
        let mut ancestor = Some(context);
        for _ in 0..MAX_DEPTH {
            let Some(id) = ancestor else { break };
            let node = self.nodes.get(id).ok_or("invalid fragment ancestor")?;
            if matches!(node.kind, NodeKind::DocumentFragment { host: Some(_) }) {
                // Template contents have an inert owner document created in
                // no-quirks mode, independent of the outer document's mode.
                mode = DocumentMode::NoQuirks;
            }
            if self.namespace(id) == Some(Namespace::Html) && self.tag(id) == Some("form") {
                form_ancestor = true;
            }
            ancestor = node.parent;
        }
        if ancestor.is_some() {
            return Err("fragment context ancestry exceeds depth limit".into());
        }
        let mut builder = TreeBuilder::new(self.scripting_enabled);
        builder.doc.mode = mode;
        builder.doc.encoding = self.encoding;
        builder.doc.url = self.url.clone();
        let html = builder.element(&HtmlToken::start("html"), false, true);
        builder.html = Some(html);
        let context_id = builder.doc.nodes.len();
        builder.doc.retained_bytes += element.retained_bytes();
        let mut context_element = element.clone();
        context_element.template_contents = None;
        builder.doc.nodes.push(Node {
            parent: None,
            children: vec![],
            kind: NodeKind::Element(context_element),
        });
        builder.doc.establish_template_contents(context_id);
        builder.fragment_context = Some(context_id);
        if element.namespace == Namespace::Html && element.tag == "template" {
            builder.template_modes.push(InsertionMode::InTemplate);
        }
        if form_ancestor {
            // This detached sentinel is not in scope, just as the real form
            // ancestor belongs to another document and is not on the stack.
            builder.form = Some(builder.doc.create_element("form"));
        }
        if element.namespace == Namespace::Html {
            let entities = matches!(element.tag.as_str(), "title" | "textarea");
            if entities
                || matches!(
                    element.tag.as_str(),
                    "style" | "xmp" | "iframe" | "noembed" | "noframes" | "script" | "plaintext"
                )
                || element.tag == "noscript" && self.scripting_enabled
            {
                // No start-tag token has been emitted in this tokenizer, so
                // there is no appropriate end tag, even for </textarea>.
                builder.raw = Some((String::new(), entities));
            }
        }
        builder.reset_mode();
        let mut document = parse_with_builder(source, builder);
        let children = std::mem::take(&mut document.nodes[html].children);
        document.nodes[html].parent = None;
        for child in &children {
            document.nodes[*child].parent = Some(document.root);
        }
        document.nodes[document.root].children = children;
        Ok(document)
    }
    pub fn scripting_enabled(&self) -> bool {
        self.scripting_enabled
    }
    pub fn url(&self) -> &url::Url {
        &self.url
    }
    pub(crate) fn set_url(&mut self, url: url::Url) {
        self.url = url;
    }
    /// Set the initial navigation URL after byte decoding, before scripts or
    /// resources run. Later same-document URL changes use set_url instead.
    pub(crate) fn initialize_url(&mut self, url: url::Url) {
        self.url = url;
        self.initialize_base_tracking();
    }
    pub fn base_url(&self) -> &url::Url {
        self.first_base.as_ref().map_or(&self.url, |(_, url)| url)
    }
    pub(crate) fn frozen_base(&self) -> Option<(NodeId, &url::Url)> {
        self.first_base.as_ref().map(|(id, url)| (*id, url))
    }
    pub(crate) fn restore_frozen_base(&mut self, id: NodeId, url: url::Url) -> Result<(), String> {
        if self.first_base_in_tree() != Some(id) || url.as_str().len() > MAX_DOM_BYTES {
            return Err("invalid frozen base identity or URL size".into());
        }
        if matches!(url.scheme(), "data" | "javascript") {
            let mut fallback = self.url.clone();
            let mut frozen = url.clone();
            fallback.set_fragment(None);
            frozen.set_fragment(None);
            if fallback != frozen {
                return Err("disallowed frozen base URL scheme".into());
            }
        }
        self.first_base = Some((id, url));
        Ok(())
    }
    fn in_document_tree(&self, mut id: NodeId) -> bool {
        for _ in 0..=MAX_DEPTH {
            if id == self.root {
                return true;
            }
            let Some(parent) = self.nodes.get(id).and_then(|node| node.parent) else {
                return false;
            };
            id = parent;
        }
        false
    }
    fn is_base_with_href(&self, id: NodeId) -> bool {
        matches!(self.nodes.get(id).map(|node| &node.kind), Some(NodeKind::Element(element)) if element.namespace == Namespace::Html && element.tag == "base" && element.attrs.contains_key("href"))
    }
    fn first_base_in_tree(&self) -> Option<NodeId> {
        let mut pending = vec![self.root];
        for _ in 0..self.nodes.len().min(MAX_NODES) {
            let id = pending.pop()?;
            let node = self.nodes.get(id)?;
            if self.is_base_with_href(id) {
                return Some(id);
            }
            // Hosted template fragments are not in the document tree.
            pending.extend(node.children.iter().rev().copied());
        }
        None
    }
    fn initialize_base_tracking(&mut self) {
        self.base_tracking = true;
        self.base_href_bytes = self
            .nodes
            .iter()
            .filter_map(|node| match &node.kind {
                NodeKind::Element(element)
                    if element.namespace == Namespace::Html && element.tag == "base" =>
                {
                    element.attrs.get("href").map(String::len)
                }
                _ => None,
            })
            .sum();
        self.first_base = None;
        self.refresh_base_url(None);
    }
    fn refresh_base_url(&mut self, changed: Option<NodeId>) {
        if !self.base_tracking {
            return;
        }
        let first = self.first_base_in_tree();
        if self.first_base.as_ref().map(|(id, _)| *id) == first
            && (first.is_none() || first != changed)
        {
            return;
        }
        self.first_base = first.map(|id| {
            let url =
                crate::document_url::parse(self, &self.url, self.attr(id, "href").unwrap_or(""))
                    .ok()
                    .filter(|url| {
                        !matches!(url.scheme(), "data" | "javascript")
                            && url.as_str().len() <= MAX_DOM_BYTES
                    })
                    .unwrap_or_else(|| self.url.clone());
            (id, url)
        });
    }
    fn base_recompute_work(&self) -> usize {
        self.nodes
            .len()
            .saturating_add(self.base_href_bytes)
            .saturating_add(self.url.as_str().len())
            .saturating_add(1)
    }
    fn subtree_contains_first_base(&self, ancestor: NodeId) -> bool {
        let Some((id, _)) = self.first_base.as_ref() else {
            return false;
        };
        let mut id = *id;
        for _ in 0..=MAX_DEPTH {
            if id == ancestor {
                return true;
            }
            let Some(parent) = self.nodes.get(id).and_then(|node| node.parent) else {
                return false;
            };
            id = parent;
        }
        false
    }
    /// Script callers discover contains_base during their existing subtree
    /// preflight, then charge this extra scan/URL work before any mutation.
    pub(crate) fn base_tree_change_work(
        &self,
        parent: NodeId,
        child: NodeId,
        contains_base: bool,
    ) -> usize {
        if self.base_tracking
            && contains_base
            && (self.in_document_tree(parent) || self.in_document_tree(child))
        {
            self.base_recompute_work()
        } else {
            0
        }
    }
    pub(crate) fn base_remove_work(&self, child: NodeId) -> usize {
        if self.base_tracking && self.subtree_contains_first_base(child) {
            self.base_recompute_work()
        } else {
            0
        }
    }
    pub(crate) fn base_clear_work(&self, parent: NodeId) -> usize {
        if self
            .first_base
            .as_ref()
            .is_some_and(|(id, _)| *id != parent)
        {
            self.base_remove_work(parent)
        } else {
            0
        }
    }
    pub(crate) fn base_attribute_work(&self, id: NodeId, name: &str) -> usize {
        if self.base_tracking
            && name.eq_ignore_ascii_case("href")
            && matches!(self.nodes.get(id).map(|node| &node.kind), Some(NodeKind::Element(element)) if element.namespace == Namespace::Html && element.tag == "base")
            && self.in_document_tree(id)
        {
            self.base_recompute_work()
        } else {
            0
        }
    }
    pub fn mode(&self) -> DocumentMode {
        self.mode
    }
    pub fn character_set(&self) -> &'static str {
        self.encoding.name()
    }
    pub(crate) fn set_encoding(&mut self, encoding: &'static encoding_rs::Encoding) {
        self.encoding = encoding;
    }
    pub(crate) fn encoding_declaration(&self) -> Option<&'static encoding_rs::Encoding> {
        self.encoding_declaration
    }
    pub fn template_contents(&self, id: NodeId) -> Option<NodeId> {
        match &self.nodes.get(id)?.kind {
            NodeKind::Element(element) => element.template_contents,
            _ => None,
        }
    }
    pub fn create_document_fragment(&mut self) -> NodeId {
        if self.nodes.len() >= MAX_NODES {
            return self.root;
        }
        let id = self.nodes.len();
        self.nodes.push(Node {
            parent: None,
            children: vec![],
            kind: NodeKind::DocumentFragment { host: None },
        });
        id
    }
    fn establish_template_contents(&mut self, id: NodeId) {
        if self.namespace(id) != Some(Namespace::Html) || self.tag(id) != Some("template") {
            return;
        }
        let content = self.create_document_fragment();
        if content == self.root {
            return;
        }
        self.nodes[content].kind = NodeKind::DocumentFragment { host: Some(id) };
        if let NodeKind::Element(element) = &mut self.nodes[id].kind {
            element.template_contents = Some(content);
        }
    }
    fn host_including_parent(&self, id: NodeId) -> Option<NodeId> {
        let node = self.nodes.get(id)?;
        node.parent.or(match node.kind {
            NodeKind::DocumentFragment { host } => host,
            _ => None,
        })
    }
    pub fn attr(&self, id: NodeId, name: &str) -> Option<&str> {
        match &self.nodes.get(id)?.kind {
            NodeKind::Element(el) if el.namespace == Namespace::Html => {
                el.attrs.get(&name.to_ascii_lowercase()).map(String::as_str)
            }
            NodeKind::Element(el) => el.attrs.get(name).map(String::as_str),
            _ => None,
        }
    }
    pub fn tag(&self, id: NodeId) -> Option<&str> {
        match &self.nodes.get(id)?.kind {
            NodeKind::Element(el) => Some(&el.tag),
            _ => None,
        }
    }
    pub fn namespace(&self, id: NodeId) -> Option<Namespace> {
        match &self.nodes.get(id)?.kind {
            NodeKind::Element(el) => Some(el.namespace),
            _ => None,
        }
    }
    pub(crate) fn text_content_shape(
        &self,
        id: NodeId,
    ) -> Result<TextContentShape<'_>, DomDataError> {
        let node = self.nodes.get(id).ok_or(DomDataError::InvalidNode)?;
        if self.nodes.len() > MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        match &node.kind {
            NodeKind::Text(text)
            | NodeKind::Comment(text)
            | NodeKind::ProcessingInstruction { data: text, .. } => {
                if !node.children.is_empty() {
                    return Err(DomDataError::InvalidData);
                }
                Ok(TextContentShape::Single(text))
            }
            NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_) => {
                if node.children.is_empty() {
                    return Ok(TextContentShape::Empty);
                }
                if let [child] = node.children.as_slice() {
                    if *child == id {
                        return Err(DomDataError::InvalidData);
                    }
                    let child = self.nodes.get(*child).ok_or(DomDataError::InvalidNode)?;
                    if let NodeKind::Text(text) = &child.kind {
                        if !child.children.is_empty() {
                            return Err(DomDataError::InvalidData);
                        }
                        return Ok(TextContentShape::Single(text));
                    }
                }
                Ok(TextContentShape::Tree)
            }
            NodeKind::Doctype(_) => {
                if !node.children.is_empty() {
                    return Err(DomDataError::InvalidData);
                }
                Ok(TextContentShape::Empty)
            }
        }
    }

    /// Bounded replacement projection for presentation. Script DOMString reads
    /// must use the exact-unit or strict-scalar helper below instead.
    pub fn text_content(&self, id: NodeId) -> String {
        let mut visits = MAX_NODES * 2;
        self.text_segments(id, &mut visits)
            .and_then(|segments| segments.scalar_output(MAX_TEXT, true, true))
            .unwrap_or_default()
    }

    // Empty/single shapes need no arena scratch. The tree fallback reserves two
    // NodeId/reference buffers and a visited byte per arena node. Script callers
    // must admit that typed scratch before entry. No callbacks occur inside.
    fn text_segments(
        &self,
        id: NodeId,
        visits_left: &mut usize,
    ) -> Result<TextSegments<'_>, DomDataError> {
        if id >= self.nodes.len() {
            return Err(DomDataError::InvalidNode);
        }
        if self.nodes.len() > MAX_NODES || *visits_left == 0 {
            return Err(DomDataError::LimitExceeded);
        }
        let shape = self.text_content_shape(id)?;
        let needed = match &shape {
            TextContentShape::Single(_)
                if matches!(
                    self.nodes[id].kind,
                    NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_)
                ) =>
            {
                3
            }
            _ => 1,
        };
        match shape {
            TextContentShape::Empty | TextContentShape::Single(_) => {
                if needed > *visits_left {
                    *visits_left = 0;
                    return Err(DomDataError::LimitExceeded);
                }
                *visits_left -= needed;
                return Ok(match shape {
                    TextContentShape::Empty => TextSegments::Empty,
                    TextContentShape::Single(text) => TextSegments::One(text),
                    TextContentShape::Tree => unreachable!(),
                });
            }
            TextContentShape::Tree => {}
        }
        let mut pending = Vec::new();
        let mut seen = Vec::new();
        let mut segments = Vec::new();
        pending
            .try_reserve_exact(self.nodes.len())
            .map_err(|_| DomDataError::AllocationFailed)?;
        seen.try_reserve_exact(self.nodes.len())
            .map_err(|_| DomDataError::AllocationFailed)?;
        segments
            .try_reserve_exact(self.nodes.len())
            .map_err(|_| DomDataError::AllocationFailed)?;
        seen.resize(self.nodes.len(), false);
        pending.push(id);
        while let Some(next) = pending.pop() {
            if *visits_left == 0 {
                return Err(DomDataError::LimitExceeded);
            }
            *visits_left -= 1;
            let Some(node) = self.nodes.get(next) else {
                return Err(DomDataError::InvalidNode);
            };
            if seen[next] {
                return Err(DomDataError::InvalidData);
            }
            seen[next] = true;
            match &node.kind {
                NodeKind::ProcessingInstruction { data, .. } if next == id => segments.push(data),
                NodeKind::Text(text) | NodeKind::Comment(text)
                    if next == id || matches!(node.kind, NodeKind::Text(_)) =>
                {
                    segments.push(text)
                }
                NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_) => {
                    if node.children.len() > *visits_left
                        || node.children.len() > self.nodes.len().saturating_sub(pending.len())
                    {
                        return Err(DomDataError::LimitExceeded);
                    }
                    *visits_left -= node.children.len();
                    pending.extend(node.children.iter().rev().copied());
                }
                _ => {}
            }
        }
        Ok(TextSegments::Many(segments))
    }

    /// Exact concatenation before scalar validation; pairs may cross Text nodes.
    pub fn text_content_scalar_bounded(
        &self,
        id: NodeId,
        maximum: usize,
        visits_left: &mut usize,
    ) -> Result<String, DomDataError> {
        let segments = self.text_segments(id, visits_left)?;
        segments.scalar_output(maximum, false, false)
    }

    /// Bounded presentation projection; refuse oversized output rather than
    /// silently treating a partial rendering source as the complete source.
    pub(crate) fn text_content_projection_bounded(
        &self,
        id: NodeId,
        maximum: usize,
        visits_left: &mut usize,
    ) -> Result<String, DomDataError> {
        let segments = self.text_segments(id, visits_left)?;
        segments.scalar_output(maximum, true, false)
    }

    pub fn text_content_units_bounded(
        &self,
        id: NodeId,
        maximum: usize,
        visits_left: &mut usize,
    ) -> Result<Vec<u16>, DomDataError> {
        let segments = self.text_segments(id, visits_left)?;
        let units = segments.as_slice().iter().flat_map(|text| text.units());
        let mut length = 0usize;
        for _ in units.clone() {
            length = length
                .checked_add(1)
                .filter(|length| *length <= maximum)
                .ok_or(DomDataError::LimitExceeded)?;
        }
        let mut output = Vec::new();
        output
            .try_reserve_exact(length)
            .map_err(|_| DomDataError::AllocationFailed)?;
        output.extend(units);
        Ok(output)
    }

    /// Direct child text for CSS input: combine units before replacing isolated
    /// surrogates. Both visits and projected UTF-8 bytes retain explicit bounds.
    pub fn child_text_content_bounded(
        &self,
        id: NodeId,
        bytes_left: &mut usize,
        child_visits_left: &mut usize,
    ) -> Option<String> {
        let node = self.nodes.get(id)?;
        let visits = node.children.len().checked_mul(2)?;
        if visits > *child_visits_left {
            *child_visits_left = 0;
            return None;
        }
        *child_visits_left -= visits;
        let units = node
            .children
            .iter()
            .filter_map(|child| match &self.nodes.get(*child)?.kind {
                NodeKind::Text(text) => Some(text.units()),
                _ => None,
            })
            .flatten();
        let output =
            string::scalar_from_units(units, (*bytes_left).min(MAX_TEXT), true, false).ok()?;
        *bytes_left -= output.len();
        Some(output)
    }
    /// HTML style's exact type check also applies to SVG style (SVG 2 §6.2).
    /// Link MIME hints have different processing and must not use this helper.
    pub(crate) fn is_css_style_element(&self, id: NodeId) -> bool {
        self.tag(id) == Some("style")
            && matches!(self.namespace(id), Some(Namespace::Html | Namespace::Svg))
            && self
                .attr(id, "type")
                .is_none_or(|kind| kind.is_empty() || kind.eq_ignore_ascii_case("text/css"))
    }
    pub fn set_text_content(&mut self, id: NodeId, text: &str) {
        if id >= self.nodes.len() || matches!(self.nodes[id].kind, NodeKind::Doctype(_)) {
            return;
        }
        let old_bytes = match &self.nodes[id].kind {
            NodeKind::Text(t)
            | NodeKind::Comment(t)
            | NodeKind::ProcessingInstruction { data: t, .. } => t.stored_bytes(),
            _ => 0,
        };
        let available = MAX_DOM_BYTES.saturating_sub(self.retained_bytes.saturating_sub(old_bytes));
        let text = &text[..floor_boundary(text, text.len().min(MAX_TEXT).min(available))];
        if let NodeKind::Text(t)
        | NodeKind::Comment(t)
        | NodeKind::ProcessingInstruction { data: t, .. } = &mut self.nodes[id].kind
        {
            self.retained_bytes = self.retained_bytes.saturating_sub(t.stored_bytes()) + text.len();
            *t = text.into();
            return;
        }
        self.clear_children(id);
        if !text.is_empty() && self.nodes.len() < MAX_NODES {
            let child = self.create_text_node(text);
            self.append_child(id, child);
        }
    }
    pub fn clear_children(&mut self, id: NodeId) {
        if id >= self.nodes.len() {
            return;
        }
        let refresh = self.base_clear_work(id) != 0;
        self.details_summaries.get_mut().remove(&id);
        let moving = self.nodes[id]
            .children
            .iter()
            .flat_map(|&child| self.moving_details(child))
            .collect::<Vec<_>>();
        for &details in &moving {
            self.forget_details_group(details);
        }
        for child in std::mem::take(&mut self.nodes[id].children) {
            if let Some(node) = self.nodes.get_mut(child) {
                node.parent = None;
            }
        }
        for details in moving {
            self.enforce_details_group(details, false);
        }
        if refresh {
            self.refresh_base_url(None);
        }
    }
    pub fn create_element(&mut self, tag: &str) -> NodeId {
        self.create_element_with_case(Namespace::Html, tag, true)
    }
    pub fn create_element_ns(&mut self, namespace: Namespace, tag: &str) -> NodeId {
        self.create_element_with_case(namespace, tag, false)
    }
    fn create_element_with_case(
        &mut self,
        namespace: Namespace,
        tag: &str,
        lowercase: bool,
    ) -> NodeId {
        if self.nodes.len() >= MAX_NODES {
            return self.root;
        }
        let mut tag = tag
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | ':' | '_'))
            .take(256)
            .collect::<String>();
        if lowercase {
            tag.make_ascii_lowercase();
        }
        if namespace == Namespace::Html && tag == "template" && self.nodes.len() + 2 > MAX_NODES {
            return self.root;
        }
        if tag.len() > MAX_DOM_BYTES.saturating_sub(self.retained_bytes) {
            return self.root;
        }
        self.retained_bytes += tag.len();
        let id = self.nodes.len();
        self.nodes.push(Node {
            parent: None,
            children: vec![],
            kind: NodeKind::Element(Element {
                namespace,
                tag,
                attrs: BTreeMap::new(),
                attr_namespaces: BTreeMap::new(),
                template_contents: None,
            }),
        });
        self.establish_template_contents(id);
        id
    }
    // Script insertion uses this admission before calling the parser-oriented
    // builder, which deliberately clamps input at its storage limits.
    pub(crate) fn admits_text_node(&self, bytes: usize) -> bool {
        self.nodes.len() < MAX_NODES
            && bytes <= MAX_TEXT
            && bytes <= MAX_DOM_BYTES.saturating_sub(self.retained_bytes)
    }
    pub fn create_text_node(&mut self, text: &str) -> NodeId {
        self.create_character_data(text, false)
    }
    pub fn create_comment(&mut self, text: &str) -> NodeId {
        self.create_character_data(text, true)
    }
    fn create_character_data(&mut self, text: &str, comment: bool) -> NodeId {
        if self.nodes.len() >= MAX_NODES {
            return self.root;
        }
        let available = MAX_DOM_BYTES.saturating_sub(self.retained_bytes);
        let text = &text[..floor_boundary(text, text.len().min(MAX_TEXT).min(available))];
        self.retained_bytes += text.len();
        let id = self.nodes.len();
        self.nodes.push(Node {
            parent: None,
            children: vec![],
            kind: if comment {
                NodeKind::Comment(text.into())
            } else {
                NodeKind::Text(text.into())
            },
        });
        id
    }
    /// Checked exact storage, moving the already admitted data buffer.
    pub(crate) fn create_text_node_owned(
        &mut self,
        data: DomString,
    ) -> Result<NodeId, DomDataError> {
        self.create_character_data_owned(data, false)
    }
    pub(crate) fn create_comment_owned(&mut self, data: DomString) -> Result<NodeId, DomDataError> {
        self.create_character_data_owned(data, true)
    }
    fn create_character_data_owned(
        &mut self,
        data: DomString,
        comment: bool,
    ) -> Result<NodeId, DomDataError> {
        let bytes = data.stored_bytes();
        if !self.admits_text_node(bytes) {
            return Err(DomDataError::LimitExceeded);
        }
        self.nodes
            .try_reserve_exact(1)
            .map_err(|_| DomDataError::AllocationFailed)?;
        let id = self.nodes.len();
        self.nodes.push(Node {
            parent: None,
            children: Vec::new(),
            kind: if comment {
                NodeKind::Comment(data)
            } else {
                NodeKind::Text(data)
            },
        });
        self.retained_bytes += bytes;
        Ok(id)
    }
    pub fn create_doctype(&mut self, mut doctype: Doctype) -> NodeId {
        if self.nodes.len() >= MAX_NODES {
            return self.root;
        }
        let mut available = MAX_DOM_BYTES
            .saturating_sub(self.retained_bytes)
            .min(MAX_TEXT);
        for value in std::iter::once(&mut doctype.name)
            .chain(doctype.public_id.iter_mut())
            .chain(doctype.system_id.iter_mut())
        {
            value.truncate(floor_boundary(value, value.len().min(available)));
            available -= value.len();
            self.retained_bytes += value.len();
        }
        let id = self.nodes.len();
        self.nodes.push(Node {
            parent: None,
            children: vec![],
            kind: NodeKind::Doctype(doctype),
        });
        id
    }
    /// Publish already converted and validated PI strings without copying or
    /// clamping. The caller must prepay any node-vector growth before entry.
    /// No author callback may run between that admission and this operation.
    pub(crate) fn create_processing_instruction_owned(
        &mut self,
        target: String,
        data: DomString,
    ) -> Result<NodeId, DomDataError> {
        let bytes = target
            .len()
            .checked_add(data.stored_bytes())
            .ok_or(DomDataError::LimitExceeded)?;
        if !self.admits_text_node(bytes) {
            return Err(DomDataError::LimitExceeded);
        }
        self.nodes
            .try_reserve_exact(1)
            .map_err(|_| DomDataError::AllocationFailed)?;
        let id = self.nodes.len();
        self.nodes.push(Node {
            parent: None,
            children: Vec::new(),
            kind: NodeKind::ProcessingInstruction { target, data },
        });
        self.retained_bytes += bytes;
        Ok(id)
    }

    /// Check replacement size before allocating its payload, without mutation.
    /// The returned ledger is advisory; publication repeats this fresh check.
    pub(crate) fn check_character_data_replacement(
        &self,
        id: NodeId,
        stored_bytes: usize,
    ) -> Result<usize, DomDataError> {
        let old = match self.nodes.get(id).map(|node| &node.kind) {
            Some(NodeKind::Text(text) | NodeKind::Comment(text))
            | Some(NodeKind::ProcessingInstruction { data: text, .. }) => text.stored_bytes(),
            _ => return Err(DomDataError::InvalidNode),
        };
        let retained = self
            .retained_bytes
            .checked_sub(old)
            .and_then(|bytes| bytes.checked_add(stored_bytes))
            .filter(|bytes| *bytes <= MAX_DOM_BYTES)
            .ok_or(DomDataError::LimitExceeded)?;
        if stored_bytes > MAX_TEXT {
            return Err(DomDataError::LimitExceeded);
        }
        Ok(retained)
    }

    /// Replace authentic CharacterData after conversion and fresh admission.
    /// Takes the owned buffer; node identity, PI target and tree links survive.
    pub(crate) fn replace_character_data(
        &mut self,
        id: NodeId,
        data: DomString,
    ) -> Result<(), DomDataError> {
        let retained = self.check_character_data_replacement(id, data.stored_bytes())?;
        let (NodeKind::Text(text)
        | NodeKind::Comment(text)
        | NodeKind::ProcessingInstruction { data: text, .. }) = &mut self.nodes[id].kind
        else {
            unreachable!("character-data kind was checked without callbacks")
        };
        *text = data;
        self.retained_bytes = retained;
        Ok(())
    }

    pub fn create_processing_instruction(&mut self, target: &str, data: &str) -> NodeId {
        if self.nodes.len() >= MAX_NODES {
            return self.root;
        }
        let available = MAX_DOM_BYTES
            .saturating_sub(self.retained_bytes)
            .min(MAX_TEXT);
        let target = &target[..floor_boundary(target, target.len().min(available))];
        let data = &data[..floor_boundary(data, data.len().min(available - target.len()))];
        self.retained_bytes += target.len() + data.len();
        let id = self.nodes.len();
        self.nodes.push(Node {
            parent: None,
            children: vec![],
            kind: NodeKind::ProcessingInstruction {
                target: target.into(),
                data: data.into(),
            },
        });
        id
    }
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        if parent >= self.nodes.len()
            || child >= self.nodes.len()
            || child == self.root
            || parent == child
            || !matches!(
                self.nodes[parent].kind,
                NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_)
            )
            || matches!(self.nodes[child].kind, NodeKind::Doctype(_))
                && !matches!(self.nodes[parent].kind, NodeKind::Document)
        {
            return;
        }
        let mut cursor = Some(parent);
        let mut ancestors = 0;
        while let Some(id) = cursor {
            if id == child || ancestors >= MAX_DEPTH {
                return;
            }
            ancestors += 1;
            cursor = self.host_including_parent(id);
        }
        let mut pending = vec![(child, 1usize)];
        let mut visited = 0;
        let mut contains_base = false;
        while let Some((id, depth)) = pending.pop() {
            visited += 1;
            if ancestors + depth > MAX_DEPTH || visited > MAX_NODES {
                return;
            }
            if let Some(n) = self.nodes.get(id) {
                contains_base |= self.is_base_with_href(id);
                pending.extend(n.children.iter().map(|n| (*n, depth + 1)));
                if let NodeKind::Element(element) = &n.kind
                    && let Some(content) = element.template_contents
                {
                    pending.push((content, depth + 1));
                }
            }
        }
        let refresh = self.base_tree_change_work(parent, child, contains_base) != 0;
        let moving = self.moving_details(child);
        for &details in &moving {
            self.forget_details_group(details);
        }
        let moved_first = self
            .subtree_contains_first_base(child)
            .then(|| self.first_base.as_ref().map(|(id, _)| *id))
            .flatten();
        if matches!(self.nodes[child].kind, NodeKind::DocumentFragment { .. }) {
            self.details_summaries.get_mut().remove(&parent);
            let children = std::mem::take(&mut self.nodes[child].children);
            for child in children {
                self.nodes[child].parent = Some(parent);
                self.nodes[parent].children.push(child);
            }
            for details in moving {
                self.enforce_details_group(details, false);
            }
            if refresh {
                self.refresh_base_url(moved_first);
            }
            return;
        }
        if let Some(old) = self.nodes[child].parent
            && let Some(n) = self.nodes.get_mut(old)
        {
            self.details_summaries.get_mut().remove(&old);
            n.children.retain(|c| *c != child);
        }
        self.details_summaries.get_mut().remove(&parent);
        self.nodes[child].parent = Some(parent);
        self.nodes[parent].children.push(child);
        for details in moving {
            self.enforce_details_group(details, false);
        }
        if refresh {
            self.refresh_base_url(moved_first);
        }
    }
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        if self.nodes.get(child).and_then(|n| n.parent) == Some(parent) {
            let refresh = self.base_remove_work(child) != 0;
            self.details_summaries.get_mut().remove(&parent);
            let moving = self.moving_details(child);
            for &details in &moving {
                self.forget_details_group(details);
            }
            self.nodes[parent].children.retain(|n| *n != child);
            self.nodes[child].parent = None;
            for details in moving {
                self.enforce_details_group(details, false);
            }
            if refresh {
                self.refresh_base_url(None);
            }
        }
    }
    pub fn set_attr(&mut self, id: NodeId, name: &str, value: &str) {
        let mut name = name[..floor_boundary(name, name.len().min(MAX_TEXT))].to_owned();
        if self.namespace(id) == Some(Namespace::Html) {
            name.make_ascii_lowercase();
        }
        let namespace = match self.nodes.get(id).map(|node| &node.kind) {
            Some(NodeKind::Element(el)) => el.attr_namespaces.get(&name).copied(),
            _ => None,
        };
        self.set_attribute(id, &name, value, namespace);
    }
    /// Set one of the namespaced attributes supported by HTML foreign-content parsing.
    pub fn set_attr_ns(
        &mut self,
        id: NodeId,
        namespace: AttributeNamespace,
        name: &str,
        value: &str,
    ) {
        if AttributeNamespace::from_qualified_name(name) == Some(namespace) {
            self.set_attribute(id, name, value, Some(namespace));
        }
    }
    fn set_attribute(
        &mut self,
        id: NodeId,
        name: &str,
        value: &str,
        namespace: Option<AttributeNamespace>,
    ) {
        let refresh = self.base_attribute_work(id, name) != 0;
        let details = namespace.is_none()
            && self.html_control_tag(id) == Some("details")
            && matches!(name, "open" | "name");
        let old_open = self.attr(id, "open").is_some();
        let old_name_size = if details && name == "name" {
            self.attr(id, "name").map_or(0, str::len)
        } else {
            0
        };
        // Preserve the index if an allocation/storage preflight rejects this
        // mutation. The previous key is removed only after a successful write.
        let old_group = details.then(|| self.details_group_key(id)).flatten();
        let mut changed = false;
        if let Some(Node {
            kind: NodeKind::Element(el),
            ..
        }) = self.nodes.get_mut(id)
        {
            if el.attrs.len() >= 1024 && !el.attrs.contains_key(name) {
                return;
            }
            let old = el.attrs.get(name);
            let old_size = old.map(|v| v.len() + name.len()).unwrap_or(0)
                + if el.attr_namespaces.contains_key(name) {
                    name.len()
                } else {
                    0
                };
            let name_size = name.len() * if namespace.is_some() { 2 } else { 1 };
            let remaining =
                MAX_DOM_BYTES.saturating_sub(self.retained_bytes.saturating_sub(old_size));
            if remaining < name_size {
                return;
            }
            let value = &value
                [..floor_boundary(value, value.len().min(MAX_TEXT).min(remaining - name_size))];
            if el.namespace == Namespace::Html && el.tag == "base" && name == "href" {
                self.base_href_bytes = self
                    .base_href_bytes
                    .saturating_sub(old.map_or(0, String::len))
                    .saturating_add(value.len());
            }
            self.retained_bytes =
                self.retained_bytes.saturating_sub(old_size) + name_size + value.len();
            el.attrs.insert(name.into(), value.into());
            changed = true;
            if let Some(namespace) = namespace {
                el.attr_namespaces.insert(name.into(), namespace);
            }
        }
        if details && changed {
            if name == "name" {
                self.details_name_bytes = self
                    .details_name_bytes
                    .saturating_sub(old_name_size)
                    .saturating_add(self.attr(id, "name").map_or(0, str::len));
            }
            if let Some(key) = old_group
                && self.details_groups.get(&key) == Some(&id)
            {
                self.details_groups.remove(&key);
            }
            let new_open = self.attr(id, "open").is_some();
            if old_open != new_open {
                self.queue_details_toggle(id, old_open, new_open);
            }
            self.enforce_details_group(id, !old_open && new_open);
        }
        if refresh {
            self.refresh_base_url(Some(id));
        }
    }
    pub fn remove_attr(&mut self, id: NodeId, name: &str) {
        let refresh = self.base_attribute_work(id, name) != 0;
        let name = if self.namespace(id) == Some(Namespace::Html) {
            name.to_ascii_lowercase()
        } else {
            name.to_owned()
        };
        let details = self.html_control_tag(id) == Some("details")
            && matches!(name.as_str(), "open" | "name");
        let old_open = self.attr(id, "open").is_some();
        if details && name == "name" {
            self.details_name_bytes = self
                .details_name_bytes
                .saturating_sub(self.attr(id, "name").map_or(0, str::len));
        }
        if details {
            self.forget_details_group(id);
        }
        if let Some(Node {
            kind: NodeKind::Element(el),
            ..
        }) = self.nodes.get_mut(id)
            && let Some(old) = el.attrs.remove(&name)
        {
            if el.namespace == Namespace::Html && el.tag == "base" && name == "href" {
                self.base_href_bytes = self.base_href_bytes.saturating_sub(old.len());
            }
            self.retained_bytes = self.retained_bytes.saturating_sub(name.len() + old.len());
            if el.attr_namespaces.remove(&name).is_some() {
                self.retained_bytes = self.retained_bytes.saturating_sub(name.len());
            }
        }
        if details {
            let new_open = self.attr(id, "open").is_some();
            if old_open != new_open {
                self.queue_details_toggle(id, old_open, new_open);
            }
            self.enforce_details_group(id, false);
        }
        if refresh {
            self.refresh_base_url(Some(id));
        }
    }
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
    pub fn query_selector(&self, selector: &str) -> Option<NodeId> {
        self.query_selector_from(self.root, selector)
    }
    pub fn query_selector_all(&self, selector: &str) -> Vec<NodeId> {
        self.query_selector_all_from(self.root, selector)
    }
    pub fn query_selector_from(&self, root: NodeId, selector: &str) -> Option<NodeId> {
        self.query_selector_impl(root, selector, true)
            .into_iter()
            .next()
    }
    pub fn query_selector_all_from(&self, root: NodeId, selector: &str) -> Vec<NodeId> {
        self.query_selector_impl(root, selector, false)
    }
    fn query_selector_impl(&self, root: NodeId, selector: &str, first: bool) -> Vec<NodeId> {
        if selector.len() > 4096 {
            return Vec::new();
        }
        let mut budget = 4_000_000usize;
        let Ok(selectors) = crate::selectors::parse_list(selector, &mut budget) else {
            return Vec::new();
        };
        let mut result = vec![];
        let mut pending = self
            .nodes
            .get(root)
            .map(|node| node.children.iter().rev().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        let mut visited = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !visited.insert(id) || visited.len() > MAX_NODES {
                continue;
            }
            if let Some(node) = self.nodes.get(id) {
                if matches!(node.kind, NodeKind::Element(_))
                    && selectors.iter().any(|selector| {
                        matches_compiled_selector(self, id, &selector.source, &mut budget)
                    })
                {
                    result.push(id);
                    if first {
                        break;
                    }
                }
                if budget == 0 {
                    break;
                }
                pending.extend(node.children.iter().rev().copied());
            }
        }
        result
    }
    pub fn stylesheets(&self) -> Vec<String> {
        let mut sources = Vec::new();
        let mut bytes_left = MAX_TEXT;
        let mut child_visits_left = MAX_STYLE_CHILD_VISITS;
        for id in self.query_selector_all("style") {
            if sources.len() >= MAX_INLINE_STYLES {
                break;
            }
            if !self.is_css_style_element(id)
                || !self.is_active_node(id)
                || self
                    .attr(id, "media")
                    .is_some_and(|media| !crate::stylesheet_loading::valid_media_condition(media))
            {
                continue;
            }
            let Some(source) =
                self.child_text_content_bounded(id, &mut bytes_left, &mut child_visits_left)
            else {
                break;
            };
            sources.push(source);
        }
        sources
    }
    /// Attached nodes outside HTML template content can contribute document metadata and resources.
    pub fn is_active_node(&self, id: NodeId) -> bool {
        let mut current = Some(id);
        for _ in 0..MAX_DEPTH {
            let Some(id) = current else {
                return false;
            };
            if id == self.root {
                return true;
            }
            if self.namespace(id) == Some(Namespace::Html) && self.tag(id) == Some("template") {
                return false;
            }
            current = self.nodes.get(id).and_then(|node| node.parent);
        }
        false
    }
    fn html_control_tag(&self, id: NodeId) -> Option<&str> {
        (self.namespace(id) == Some(Namespace::Html))
            .then(|| self.tag(id))
            .flatten()
    }
    /// The first direct HTML summary child remains the disclosure summary even
    /// when author CSS hides it. Foreign summaries and nested summaries do not.
    pub fn first_summary(&self, details: NodeId) -> Option<NodeId> {
        if self.html_control_tag(details) != Some("details") {
            return None;
        }
        if let Some(summary) = self.details_summaries.borrow().get(&details) {
            return *summary;
        }
        let summary = self
            .nodes
            .get(details)?
            .children
            .iter()
            .copied()
            .find(|&id| self.html_control_tag(id) == Some("summary"));
        self.details_summaries.borrow_mut().insert(details, summary);
        summary
    }
    pub fn summary_details(&self, summary: NodeId) -> Option<NodeId> {
        if self.html_control_tag(summary) != Some("summary") {
            return None;
        }
        let parent = self.nodes.get(summary)?.parent?;
        (self.first_summary(parent) == Some(summary)).then_some(parent)
    }
    /// Rendering/native eligibility, deliberately separate from document
    /// activity: hidden disclosure content can still run scripts and submit.
    pub fn disclosure_hidden(&self, mut node: NodeId) -> bool {
        for _ in 0..=MAX_DEPTH {
            let Some(parent) = self.nodes.get(node).and_then(|n| n.parent) else {
                return false;
            };
            if self.html_control_tag(parent) == Some("details")
                && self.attr(parent, "open").is_none()
                && self.first_summary(parent) != Some(node)
            {
                return true;
            }
            node = parent;
        }
        true
    }
    /// One linear traversal for renderers; do not repeatedly scan large sibling
    /// lists while laying out each descendant of a closed disclosure.
    pub fn disclosure_hidden_mask(&self) -> Vec<bool> {
        let mut hidden = vec![false; self.nodes.len()];
        let mut pending = vec![(self.root, false)];
        let mut visits = 0;
        while let Some((id, ancestor_hidden)) = pending.pop() {
            if visits >= MAX_NODES {
                break;
            }
            visits += 1;
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            hidden[id] = ancestor_hidden;
            let closed =
                self.html_control_tag(id) == Some("details") && self.attr(id, "open").is_none();
            let summary = closed.then(|| self.first_summary(id)).flatten();
            pending.extend(
                node.children
                    .iter()
                    .rev()
                    .map(|&child| (child, ancestor_hidden || closed && Some(child) != summary)),
            );
        }
        hidden
    }
    fn details_root(&self, mut id: NodeId) -> NodeId {
        for _ in 0..=MAX_DEPTH {
            let Some(parent) = self.nodes.get(id).and_then(|n| n.parent) else {
                break;
            };
            id = parent;
        }
        id
    }
    fn details_group_key(&self, id: NodeId) -> Option<(NodeId, String)> {
        if self.html_control_tag(id) != Some("details") || self.attr(id, "open").is_none() {
            return None;
        }
        let name = self.attr(id, "name").filter(|name| !name.is_empty())?;
        Some((self.details_root(id), name.to_owned()))
    }
    fn forget_details_group(&mut self, id: NodeId) {
        if let Some(key) = self.details_group_key(id)
            && self.details_groups.get(&key) == Some(&id)
        {
            self.details_groups.remove(&key);
        }
    }
    fn enforce_details_group(&mut self, id: NodeId, opening: bool) {
        let Some(key) = self.details_group_key(id) else {
            return;
        };
        if let Some(other) = self
            .details_groups
            .get(&key)
            .copied()
            .filter(|&other| other != id)
        {
            if opening {
                self.remove_attr(other, "open");
            } else {
                self.remove_attr(id, "open");
                return;
            }
        }
        self.details_groups.insert(key, id);
    }
    fn moving_details(&self, root: NodeId) -> Vec<NodeId> {
        if self.details_groups.is_empty() {
            return Vec::new();
        }
        let mut result = Vec::new();
        let mut pending = vec![root];
        let mut visited = 0;
        while let Some(id) = pending.pop() {
            if visited >= MAX_NODES {
                break;
            }
            visited += 1;
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if self.html_control_tag(id) == Some("details")
                && self.attr(id, "open").is_some()
                && self.attr(id, "name").is_some_and(|name| !name.is_empty())
            {
                result.push(id);
            }
            // Hosted template fragments have separate DOM roots and do not
            // change their name groups when their host is moved.
            pending.extend(node.children.iter().rev().copied());
        }
        result
    }
    fn queue_details_toggle(&mut self, node: NodeId, old_open: bool, new_open: bool) {
        let old_open = if let Some(tracker) = self.details_trackers.remove(&node) {
            // The referenced task may already be running. Its old state still
            // participates in coalescing even though it is no longer queued.
            self.details_toggles.remove(&tracker.task);
            tracker.old_open
        } else {
            old_open
        };
        let sequence = self.details_sequence;
        self.details_sequence = self.details_sequence.saturating_add(1);
        self.details_trackers.insert(
            node,
            DetailsToggleTracker {
                task: sequence,
                old_open,
            },
        );
        self.details_toggles.insert(
            sequence,
            DetailsToggle {
                node,
                old_open,
                new_open,
            },
        );
    }
    pub(crate) fn peek_details_toggle(&self) -> Option<DetailsToggleTask> {
        self.details_toggles
            .first_key_value()
            .map(|(&id, &event)| DetailsToggleTask { id, event })
    }
    /// Start only the expected FIFO task, keeping the element's tracker intact.
    /// An untracked task must not replace a newer task's tracker on entry.
    pub(crate) fn begin_details_toggle(&mut self, expected_id: u64) -> Option<DetailsToggleTask> {
        if self.details_active.is_some() {
            return None;
        }
        let task = self.peek_details_toggle()?;
        if task.id != expected_id {
            return None;
        }
        self.details_toggles.pop_first();
        self.details_active = Some(task);
        Some(task)
    }
    /// Finishing clears the element's tracker even if a callback replaced it.
    /// It never cancels the queued task referenced by that replacement tracker.
    pub(crate) fn finish_details_toggle(&mut self, id: u64) -> bool {
        let Some(task) = self.details_active.filter(|task| task.id == id) else {
            return false;
        };
        self.details_trackers.remove(&task.event.node);
        self.details_active = None;
        true
    }
    pub(crate) fn details_toggle_running(&self) -> bool {
        self.details_active.is_some()
    }
    pub(crate) fn has_pending_details_toggles(&self) -> bool {
        !self.details_toggles.is_empty()
    }
    pub(crate) fn details_attribute_work(&self, id: NodeId, name: &str) -> usize {
        if self.html_control_tag(id) != Some("details")
            || !(name.eq_ignore_ascii_case("open") || name.eq_ignore_ascii_case("name"))
        {
            return 0;
        }
        // Include worst-case ordered-map name comparisons and old-value copies.
        self.details_name_bytes
            .saturating_mul(64)
            .saturating_add(MAX_DEPTH * 4)
    }
    pub(crate) fn details_tree_change_work(&self, _parent: NodeId, _child: NodeId) -> usize {
        if self.details_groups.is_empty() {
            return 0;
        }
        self.nodes
            .len()
            .saturating_mul(MAX_DEPTH + 8)
            .saturating_add(self.details_name_bytes.saturating_mul(64))
    }
    pub(crate) fn details_bulk_change_work(
        &self,
        new_nodes: usize,
        new_name_bytes: usize,
    ) -> usize {
        if self.details_groups.is_empty() && new_name_bytes == 0 {
            return 0;
        }
        self.nodes
            .len()
            .saturating_add(new_nodes)
            .saturating_mul(MAX_DEPTH + 8)
            .saturating_add(
                self.details_name_bytes
                    .saturating_add(new_name_bytes)
                    .saturating_mul(64),
            )
    }
    fn is_descendant_of(&self, mut node: NodeId, ancestor: NodeId) -> bool {
        for _ in 0..crate::dom::MAX_DEPTH {
            if node == ancestor {
                return true;
            }
            let Some(parent) = self.nodes.get(node).and_then(|node| node.parent) else {
                return false;
            };
            node = parent;
        }
        false
    }
    pub fn disabled_control(&self, node: NodeId) -> bool {
        let tag = self.html_control_tag(node).unwrap_or("");
        if !matches!(
            tag,
            "input" | "button" | "select" | "textarea" | "option" | "optgroup"
        ) {
            return false;
        }
        if self.attr(node, "disabled").is_some() {
            return true;
        }
        let mut ancestor = self.nodes.get(node).and_then(|node| node.parent);
        for _ in 0..crate::dom::MAX_DEPTH {
            let Some(id) = ancestor else {
                break;
            };
            if tag == "option"
                && self.html_control_tag(id) == Some("optgroup")
                && self.attr(id, "disabled").is_some()
            {
                return true;
            }
            if self.html_control_tag(id) == Some("fieldset") && self.attr(id, "disabled").is_some()
            {
                let first_legend = self.nodes[id]
                    .children
                    .iter()
                    .copied()
                    .find(|&child| self.html_control_tag(child) == Some("legend"));
                if !first_legend.is_some_and(|legend| self.is_descendant_of(node, legend)) {
                    return true;
                }
            }
            ancestor = self.nodes.get(id).and_then(|node| node.parent);
        }
        false
    }
    pub fn interaction_blocked(&self, node: NodeId) -> bool {
        if !self.is_active_node(node) {
            return true;
        }
        let mut ancestor = Some(node);
        for _ in 0..crate::dom::MAX_DEPTH {
            let Some(id) = ancestor else {
                return false;
            };
            if self.attr(id, "inert").is_some() || self.disabled_control(id) {
                return true;
            }
            ancestor = self.nodes.get(id).and_then(|node| node.parent);
        }
        true
    }
    pub fn can_edit_control(&self, node: NodeId) -> bool {
        if self.interaction_blocked(node)
            || self.disclosure_hidden(node)
            || self.attr(node, "readonly").is_some()
        {
            return false;
        }
        match self.html_control_tag(node) {
            Some("textarea") => true,
            Some("input") => !matches!(
                self.attr(node, "type")
                    .unwrap_or("text")
                    .to_ascii_lowercase()
                    .as_str(),
                "checkbox"
                    | "radio"
                    | "submit"
                    | "button"
                    | "reset"
                    | "hidden"
                    | "file"
                    | "range"
                    | "color"
            ),
            _ => false,
        }
    }
    /// DOM policy for the native UI's supported focusable controls; CSS visibility is separate.
    pub fn can_focus_control(&self, node: NodeId) -> bool {
        if self.interaction_blocked(node) || self.disclosure_hidden(node) {
            return false;
        }
        match self.html_control_tag(node) {
            Some("input") => !self
                .attr(node, "type")
                .is_some_and(|kind| kind.eq_ignore_ascii_case("hidden")),
            Some("textarea" | "button") => true,
            Some("a") => self.attr(node, "href").is_some(),
            Some("summary") => self.summary_details(node).is_some(),
            Some("details") => self.first_summary(node).is_none(),
            _ => false,
        }
    }
    pub fn title(&self) -> String {
        // Host metadata has its own structural/output bounds, not a fresh
        // author execution allowance. Page retains its empty-title URL fallback.
        let mut budget = DomMutationBudget {
            steps: title::HOST_TITLE_WORK,
            allocated: 0,
            heap_limit: usize::MAX,
        };
        self.title_projection_bounded(MAX_TEXT, &mut budget)
            .unwrap_or_default()
    }
    /// Find document metadata without scanning past the first applicable HTML element.
    pub fn first_html_element(&self, tag: &str) -> Option<NodeId> {
        let mut pending = vec![self.root];
        let mut visited = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !visited.insert(id) || visited.len() > MAX_NODES {
                continue;
            }
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if let NodeKind::Element(element) = &node.kind
                && element.namespace == Namespace::Html
            {
                if element.tag == "template" {
                    continue;
                }
                if element.tag == tag {
                    return Some(id);
                }
            }
            pending.extend(node.children.iter().rev().copied());
        }
        None
    }
    /// Serialize the attached subtree without recursion, retaining at most 8 MiB.
    pub fn outer_html(&self, id: NodeId) -> String {
        let mut out = SerializedText::default();
        let mut pending = vec![(id, false)];
        let mut visited = BTreeSet::new();
        while let Some((id, closing)) = pending.pop() {
            if out.failed {
                break;
            }
            if out.len() >= MAX_TEXT {
                break;
            }
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if closing {
                if let NodeKind::Element(element) = &node.kind {
                    push_serialized(&mut out, "</");
                    push_serialized(&mut out, &element.tag);
                    push_serialized(&mut out, ">");
                }
                continue;
            }
            if !visited.insert(id) || visited.len() > MAX_NODES {
                continue;
            }
            match &node.kind {
                NodeKind::Document | NodeKind::DocumentFragment { .. } => {
                    pending.extend(node.children.iter().rev().map(|id| (*id, false)))
                }
                NodeKind::Comment(text) => {
                    push_serialized(&mut out, "<!--");
                    out.push_units(text.units());
                    push_serialized(&mut out, "-->");
                }
                NodeKind::Doctype(doctype) => {
                    push_serialized(&mut out, "<!DOCTYPE ");
                    push_serialized(&mut out, &doctype.name);
                    push_serialized(&mut out, ">");
                }
                NodeKind::ProcessingInstruction { target, data } => {
                    push_serialized(&mut out, "<?");
                    push_serialized(&mut out, target);
                    push_serialized(&mut out, " ");
                    out.push_units(data.units());
                    push_serialized(&mut out, "?>");
                }
                NodeKind::Text(text) => {
                    if node.parent.and_then(|parent| self.namespace(parent))
                        == Some(Namespace::Html)
                        && node
                            .parent
                            .and_then(|parent| self.tag(parent))
                            .is_some_and(|tag| {
                                tag == "noscript" && self.scripting_enabled
                                    || matches!(
                                        tag,
                                        "script"
                                            | "style"
                                            | "xmp"
                                            | "iframe"
                                            | "noembed"
                                            | "noframes"
                                            | "plaintext"
                                    )
                            })
                    {
                        out.push_units(text.units());
                    } else {
                        escape_serialized_units(&mut out, text.units(), false);
                    }
                }
                NodeKind::Element(element) => {
                    push_serialized(&mut out, "<");
                    push_serialized(&mut out, &element.tag);
                    for (name, value) in &element.attrs {
                        if name.chars().any(|c| {
                            c.is_ascii_whitespace()
                                || matches!(c, '\'' | '"' | '<' | '>' | '/' | '=' | '\0')
                        }) {
                            continue;
                        }
                        push_serialized(&mut out, " ");
                        push_serialized(&mut out, name);
                        push_serialized(&mut out, "=\"");
                        escape_serialized(&mut out, value, true);
                        push_serialized(&mut out, "\"");
                        if out.len() >= MAX_TEXT {
                            break;
                        }
                    }
                    push_serialized(&mut out, ">");
                    if element.namespace != Namespace::Html || !is_void(&element.tag) {
                        pending.push((id, true));
                        let children = element
                            .template_contents
                            .and_then(|id| self.nodes.get(id))
                            .map_or(&node.children, |fragment| &fragment.children);
                        pending.extend(children.iter().rev().map(|id| (*id, false)));
                    }
                }
            }
        }
        out.finish()
    }
    fn push_text(&mut self, parent: NodeId, text: String) {
        let mut text = text;
        let available = MAX_DOM_BYTES.saturating_sub(self.retained_bytes);
        text.truncate(floor_boundary(&text, text.len().min(available)));
        if text.is_empty() {
            return;
        }
        if let Some(last) = self.nodes[parent].children.last().copied()
            && let NodeKind::Text(s) = &mut self.nodes[last].kind
            && let Some(s) = s.scalar_mut()
        {
            self.retained_bytes += text.len();
            s.push_str(&text);
            return;
        }
        if self.nodes.len() < MAX_NODES {
            self.retained_bytes += text.len();
            let id = self.nodes.len();
            self.nodes.push(Node {
                parent: Some(parent),
                children: vec![],
                kind: NodeKind::Text(text.into()),
            });
            self.nodes[parent].children.push(id);
        }
    }
}
/// Bounded presentation output with one pending high unit. Keeping that state
/// across pieces preserves a pair split across adjacent Text nodes; markup
/// naturally ends a pending pair. No exact source payload is changed.
#[derive(Default)]
struct SerializedText {
    output: String,
    high: Option<u16>,
    failed: bool,
}
impl SerializedText {
    fn len(&self) -> usize {
        self.output.len()
    }
    fn emit(&mut self, value: char) -> bool {
        if self.failed || value.len_utf8() > MAX_TEXT.saturating_sub(self.output.len()) {
            self.failed = true;
            return false;
        }
        if self.output.try_reserve(value.len_utf8()).is_err() {
            self.failed = true;
            return false;
        }
        self.output.push(value);
        true
    }
    fn unit(&mut self, unit: u16) -> bool {
        if let Some(high) = self.high.take() {
            if (0xdc00..=0xdfff).contains(&unit) {
                let value = 0x10000 + (((high as u32) - 0xd800) << 10) + (unit as u32 - 0xdc00);
                return self.emit(char::from_u32(value).unwrap());
            }
            if !self.emit(char::REPLACEMENT_CHARACTER) {
                return false;
            }
        }
        match unit {
            0xd800..=0xdbff => {
                self.high = Some(unit);
                true
            }
            0xdc00..=0xdfff => self.emit(char::REPLACEMENT_CHARACTER),
            _ => self.emit(char::from_u32(unit as u32).unwrap()),
        }
    }
    fn push_units(&mut self, units: impl Iterator<Item = u16>) {
        for unit in units {
            if !self.unit(unit) {
                break;
            }
        }
    }
    fn finish(mut self) -> String {
        if self.high.take().is_some() {
            self.emit(char::REPLACEMENT_CHARACTER);
        }
        self.output
    }
}
fn push_serialized(out: &mut SerializedText, text: &str) {
    out.push_units(text.encode_utf16());
}
fn escape_serialized(out: &mut SerializedText, text: &str, attribute: bool) {
    escape_serialized_units(out, text.encode_utf16(), attribute);
}
fn escape_serialized_units(
    out: &mut SerializedText,
    units: impl Iterator<Item = u16>,
    attribute: bool,
) {
    for unit in units {
        if out.len() >= MAX_TEXT {
            break;
        }
        match unit {
            38 => push_serialized(out, "&amp;"),
            0xa0 => push_serialized(out, "&nbsp;"),
            60 => push_serialized(out, "&lt;"),
            62 => push_serialized(out, "&gt;"),
            34 if attribute => push_serialized(out, "&quot;"),
            _ => {
                if !out.unit(unit) {
                    break;
                }
            }
        }
    }
}
fn floor_boundary(s: &str, mut i: usize) -> usize {
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}
fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0c)
}
fn is_void(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "basefont"
            | "bgsound"
            | "br"
            | "col"
            | "embed"
            | "frame"
            | "hr"
            | "img"
            | "input"
            | "keygen"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
fn closes_p(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "center"
            | "details"
            | "dialog"
            | "dir"
            | "div"
            | "dl"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hgroup"
            | "hr"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "listing"
            | "plaintext"
            | "search"
            | "section"
            | "summary"
            | "table"
            | "ul"
    )
}

#[derive(Debug)]
enum HtmlToken {
    Characters(String),
    Comment(String),
    Doctype(Doctype),
    ProcessingInstruction {
        target: String,
        data: String,
    },
    Start {
        tag: String,
        attrs: BTreeMap<String, String>,
        self_closing: bool,
    },
    End(String),
    Eof,
}
impl HtmlToken {
    fn start(tag: &str) -> Self {
        Self::Start {
            tag: tag.into(),
            attrs: BTreeMap::new(),
            self_closing: false,
        }
    }
    fn tag(&self) -> &str {
        match self {
            Self::Start { tag, .. } | Self::End(tag) => tag,
            _ => "",
        }
    }
    fn is_start(&self) -> bool {
        matches!(self, Self::Start { .. })
    }
    fn is_end(&self) -> bool {
        matches!(self, Self::End(_))
    }
    fn whitespace(&self) -> bool {
        matches!(self, Self::Characters(text) if text.bytes().all(is_space))
    }
}

struct HtmlTokenizer<'a> {
    source: &'a str,
    position: usize,
    raw: Option<(String, bool)>,
    foreign: bool,
    foreign_characters: bool,
}
impl<'a> HtmlTokenizer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            raw: None,
            foreign: false,
            foreign_characters: false,
        }
    }
    fn next(&mut self) -> HtmlToken {
        let bytes = self.source.as_bytes();
        if let Some((tag, entities)) = self.raw.take() {
            let start = self.position;
            let close = format!("</{tag}");
            let mut script_escape = 0u8;
            if tag.is_empty() {
                // Fragment raw-text contexts and plaintext never recognize an
                // end tag. Do not use a magic tag spelling that input can forge.
                self.position = bytes.len();
            }
            while self.position < bytes.len() {
                if tag == "script" {
                    if script_escape == 0 && bytes[self.position..].starts_with(b"<!--") {
                        script_escape = 1;
                        self.position += 4;
                        continue;
                    }
                    if script_escape != 0 && bytes[self.position..].starts_with(b"-->") {
                        script_escape = 0;
                        self.position += 3;
                        continue;
                    }
                    let marker = if script_escape == 1 {
                        "<script"
                    } else {
                        "</script"
                    };
                    if script_escape != 0
                        && self
                            .source
                            .get(self.position..self.position + marker.len())
                            .is_some_and(|s| s.eq_ignore_ascii_case(marker))
                        && bytes
                            .get(self.position + marker.len())
                            .is_some_and(|b| is_space(*b) || matches!(b, b'/' | b'>'))
                    {
                        script_escape = if script_escape == 1 { 2 } else { 1 };
                        self.position += marker.len();
                        continue;
                    }
                }
                if script_escape != 2
                    && bytes[self.position] == b'<'
                    && self
                        .source
                        .get(self.position..self.position.saturating_add(close.len()))
                        .is_some_and(|text| text.eq_ignore_ascii_case(&close))
                    && bytes
                        .get(self.position + close.len())
                        .is_some_and(|b| is_space(*b) || matches!(b, b'>' | b'/'))
                {
                    break;
                }
                self.position += 1;
            }
            if self.position != start {
                let raw = &self.source[start..self.position];
                return HtmlToken::Characters(if entities {
                    decode_entities(raw)
                } else {
                    raw.replace('\0', "\u{fffd}")
                });
            }
        }
        if self.position >= bytes.len() {
            return HtmlToken::Eof;
        }
        let start = self.position;
        if bytes[start] != b'<' {
            while self.position < bytes.len() && bytes[self.position] != b'<' {
                self.position += 1;
            }
            let data = &self.source[start..self.position];
            return HtmlToken::Characters(if self.foreign_characters {
                decode_entities_in_state(data, false, false)
            } else {
                decode_entities(&data.replace('\0', ""))
            });
        }
        if self.source[start..].starts_with("<!--") {
            self.position += 4;
            return HtmlToken::Comment(self.comment());
        }
        if self
            .source
            .get(start..start + 9)
            .is_some_and(|text| text.eq_ignore_ascii_case("<!doctype"))
        {
            self.position += 9;
            return HtmlToken::Doctype(self.doctype());
        }
        if self.foreign && self.source[start..].starts_with("<![CDATA[") {
            let begin = start + 9;
            let end = self.source[begin..]
                .find("]]>")
                .map(|offset| begin + offset)
                .unwrap_or(bytes.len());
            self.position = (end + 3).min(bytes.len());
            return HtmlToken::Characters(if self.foreign_characters {
                self.source[begin..end].to_owned()
            } else {
                self.source[begin..end].replace('\0', "")
            });
        }
        if self.source[start..].starts_with("<?") {
            self.position += 2;
            return self.processing_instruction();
        }
        if self.source[start..].starts_with("<!") {
            self.position += 2;
            return HtmlToken::Comment(self.bogus_comment());
        }
        let closing = bytes.get(start + 1) == Some(&b'/');
        let mut p = start + if closing { 2 } else { 1 };
        if !bytes.get(p).is_some_and(u8::is_ascii_alphabetic) {
            if closing && bytes.get(p) == Some(&b'>') {
                self.position = p + 1;
                return HtmlToken::Characters(String::new());
            }
            if closing && p < bytes.len() {
                self.position = p;
                return HtmlToken::Comment(self.bogus_comment());
            }
            self.position += 1;
            return HtmlToken::Characters("<".into());
        }
        let begin = p;
        while p < bytes.len() && !is_space(bytes[p]) && !matches!(bytes[p], b'/' | b'>') {
            p += 1;
        }
        let tag = self.source[begin..p]
            .replace('\0', "\u{fffd}")
            .to_ascii_lowercase();
        let mut attrs = BTreeMap::new();
        let mut self_closing = false;
        let mut complete = false;
        while p < bytes.len() {
            while p < bytes.len() && is_space(bytes[p]) {
                p += 1;
            }
            if p >= bytes.len() {
                break;
            }
            if bytes[p] == b'>' {
                p += 1;
                complete = true;
                break;
            }
            if bytes[p] == b'/' {
                p += 1;
                if bytes.get(p) == Some(&b'>') {
                    p += 1;
                    self_closing = true;
                    complete = true;
                    break;
                }
                continue;
            }
            let begin = p;
            if bytes[p] == b'=' {
                p += 1;
            }
            while p < bytes.len() && !is_space(bytes[p]) && !matches!(bytes[p], b'=' | b'>' | b'/')
            {
                p += 1;
            }
            let name = self.source[begin..p]
                .replace('\0', "\u{fffd}")
                .to_ascii_lowercase();
            while p < bytes.len() && is_space(bytes[p]) {
                p += 1;
            }
            let mut value = String::new();
            if bytes.get(p) == Some(&b'=') {
                p += 1;
                while p < bytes.len() && is_space(bytes[p]) {
                    p += 1;
                }
                if let Some(quote @ (b'\'' | b'"')) = bytes.get(p).copied() {
                    p += 1;
                    let begin = p;
                    while p < bytes.len() && bytes[p] != quote {
                        p += 1;
                    }
                    value = decode_entities_context(&self.source[begin..p], true);
                    if p < bytes.len() {
                        p += 1;
                    }
                } else {
                    let begin = p;
                    while p < bytes.len() && !is_space(bytes[p]) && bytes[p] != b'>' {
                        p += 1;
                    }
                    value = decode_entities_context(&self.source[begin..p], true);
                }
            }
            if attrs.len() < 1024 {
                attrs.entry(name).or_insert(value);
            }
        }
        self.position = p;
        if !complete {
            return HtmlToken::Eof;
        }
        if closing {
            HtmlToken::End(tag)
        } else {
            HtmlToken::Start {
                tag,
                attrs,
                self_closing,
            }
        }
    }
    fn bogus_comment(&mut self) -> String {
        let start = self.position;
        let end = self.source[start..]
            .find('>')
            .map(|offset| start + offset)
            .unwrap_or(self.source.len());
        self.position = (end + 1).min(self.source.len());
        self.source[start..end].replace('\0', "\u{fffd}")
    }
    fn processing_instruction(&mut self) -> HtmlToken {
        let bytes = self.source.as_bytes();
        let begin = self.position;
        if self.position == bytes.len() {
            return HtmlToken::Eof;
        }
        if !bytes[self.position].is_ascii_alphabetic() && bytes[self.position] != b'_' {
            self.position = begin - 1;
            return HtmlToken::Comment(self.bogus_comment());
        }
        while self.position < bytes.len()
            && (bytes[self.position].is_ascii_alphanumeric()
                || matches!(bytes[self.position], b'-' | b'_'))
        {
            self.position += 1;
        }
        if self.position == bytes.len() {
            return HtmlToken::Eof;
        }
        let target = &self.source[begin..self.position];
        if !(is_space(bytes[self.position]) || matches!(bytes[self.position], b'?' | b'>'))
            || target.eq_ignore_ascii_case("xml")
            || target.eq_ignore_ascii_case("xml-stylesheet")
        {
            self.position = begin - 1;
            return HtmlToken::Comment(self.bogus_comment());
        }
        while bytes.get(self.position).is_some_and(|b| is_space(*b)) {
            self.position += 1;
        }
        let data_begin = self.position;
        let Some(offset) = self.source[data_begin..].find('>') else {
            self.position = bytes.len();
            return HtmlToken::Eof;
        };
        let end = data_begin + offset;
        let data_end = if end > data_begin && bytes[end - 1] == b'?' {
            end - 1
        } else {
            end
        };
        self.position = end + 1;
        HtmlToken::ProcessingInstruction {
            target: target.into(),
            data: self.source[data_begin..data_end].into(),
        }
    }
    fn comment(&mut self) -> String {
        // These states mirror the comment tokenizer, including abrupt and nested endings.
        #[derive(Clone, Copy)]
        enum State {
            Start,
            StartDash,
            Data,
            Less,
            Bang,
            BangDash,
            BangDashDash,
            EndDash,
            End,
            EndBang,
        }
        let mut state = State::Start;
        let mut out = String::new();
        while self.position < self.source.len() {
            let c = self.source[self.position..]
                .chars()
                .next()
                .unwrap_or('\u{fffd}');
            let mut consume = true;
            match state {
                State::Start => match c {
                    '-' => state = State::StartDash,
                    '>' => {
                        self.position += 1;
                        break;
                    }
                    _ => {
                        state = State::Data;
                        consume = false;
                    }
                },
                State::StartDash => match c {
                    '-' => state = State::End,
                    '>' => {
                        self.position += 1;
                        break;
                    }
                    _ => {
                        out.push('-');
                        state = State::Data;
                        consume = false;
                    }
                },
                State::Data => match c {
                    '<' => {
                        out.push('<');
                        state = State::Less;
                    }
                    '-' => state = State::EndDash,
                    '\0' => out.push('\u{fffd}'),
                    _ => out.push(c),
                },
                State::Less => match c {
                    '!' => {
                        out.push('!');
                        state = State::Bang;
                    }
                    '<' => out.push('<'),
                    _ => {
                        state = State::Data;
                        consume = false;
                    }
                },
                State::Bang => {
                    if c == '-' {
                        state = State::BangDash;
                    } else {
                        state = State::Data;
                        consume = false;
                    }
                }
                State::BangDash => {
                    if c == '-' {
                        state = State::BangDashDash;
                    } else {
                        state = State::EndDash;
                        consume = false;
                    }
                }
                State::BangDashDash => {
                    state = State::End;
                    consume = false;
                }
                State::EndDash => {
                    if c == '-' {
                        state = State::End;
                    } else {
                        out.push('-');
                        state = State::Data;
                        consume = false;
                    }
                }
                State::End => match c {
                    '>' => {
                        self.position += 1;
                        break;
                    }
                    '!' => state = State::EndBang,
                    '-' => out.push('-'),
                    _ => {
                        out.push_str("--");
                        state = State::Data;
                        consume = false;
                    }
                },
                State::EndBang => match c {
                    '-' => {
                        out.push_str("--!");
                        state = State::EndDash;
                    }
                    '>' => {
                        self.position += 1;
                        break;
                    }
                    _ => {
                        out.push_str("--!");
                        state = State::Data;
                        consume = false;
                    }
                },
            }
            if consume {
                self.position += c.len_utf8();
            }
        }
        out
    }
    fn doctype(&mut self) -> Doctype {
        let bytes = self.source.as_bytes();
        let mut result = Doctype {
            name: String::new(),
            public_id: None,
            system_id: None,
            force_quirks: false,
        };
        while bytes.get(self.position).is_some_and(|b| is_space(*b)) {
            self.position += 1;
        }
        let start = self.position;
        while self.position < bytes.len()
            && !is_space(bytes[self.position])
            && bytes[self.position] != b'>'
        {
            self.position += 1;
        }
        result.name = self.source[start..self.position]
            .replace('\0', "\u{fffd}")
            .to_ascii_lowercase();
        if result.name.is_empty() {
            result.force_quirks = true;
        }
        while bytes.get(self.position).is_some_and(|b| is_space(*b)) {
            self.position += 1;
        }
        if bytes.get(self.position) == Some(&b'>') {
            self.position += 1;
            return result;
        }
        if self.position == bytes.len() {
            result.force_quirks = true;
            return result;
        }
        let public = self
            .source
            .get(self.position..self.position + 6)
            .is_some_and(|s| s.eq_ignore_ascii_case("public"));
        let system = self
            .source
            .get(self.position..self.position + 6)
            .is_some_and(|s| s.eq_ignore_ascii_case("system"));
        if !(public || system) {
            result.force_quirks = true;
            self.bogus_comment();
            return result;
        }
        self.position += 6;
        for identifier in 0..if public { 2 } else { 1 } {
            while bytes.get(self.position).is_some_and(|b| is_space(*b)) {
                self.position += 1;
            }
            if public && identifier == 1 && bytes.get(self.position) == Some(&b'>') {
                self.position += 1;
                return result;
            }
            let Some(quote @ (b'\'' | b'"')) = bytes.get(self.position).copied() else {
                result.force_quirks = true;
                self.bogus_comment();
                return result;
            };
            self.position += 1;
            let begin = self.position;
            while self.position < bytes.len()
                && bytes[self.position] != quote
                && bytes[self.position] != b'>'
            {
                self.position += 1;
            }
            let value = Some(self.source[begin..self.position].replace('\0', "\u{fffd}"));
            if public && identifier == 0 {
                result.public_id = value;
            } else {
                result.system_id = value;
            }
            if bytes.get(self.position) != Some(&quote) {
                result.force_quirks = true;
                if self.position < bytes.len() {
                    self.position += 1;
                }
                return result;
            }
            self.position += 1;
        }
        while bytes.get(self.position).is_some_and(|b| is_space(*b)) {
            self.position += 1;
        }
        if self.position == bytes.len() {
            result.force_quirks = true;
        }
        self.bogus_comment();
        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    InTable,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InTemplate,
    InFrameset,
    AfterFrameset,
    AfterAfterFrameset,
    AfterBody,
    AfterAfterBody,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    Regular,
    Button,
    ListItem,
    Table,
}
struct TreeBuilder {
    doc: Document,
    stack: Vec<NodeId>,
    // Entries refer to immutable parser-created attributes; None is a scope marker.
    formatting: Vec<Option<NodeId>>,
    html: Option<NodeId>,
    head: Option<NodeId>,
    body: Option<NodeId>,
    form: Option<NodeId>,
    fragment_context: Option<NodeId>,
    mode: InsertionMode,
    template_modes: Vec<InsertionMode>,
    scripting: bool,
    frameset_ok: bool,
    raw: Option<(String, bool)>,
    raw_node: Option<(NodeId, InsertionMode)>,
    pending_table_text: String,
    skip_lf: bool,
    // Parser-time selectedcontent connections; disabled primary entries remain
    // present so a later non-primary element cannot take over their identity.
    selected_contents: BTreeMap<NodeId, Option<NodeId>>,
    work: usize,
}
impl TreeBuilder {
    fn new(scripting: bool) -> Self {
        Self {
            doc: Document {
                nodes: vec![Node {
                    parent: None,
                    children: vec![],
                    kind: NodeKind::Document,
                }],
                root: 0,
                retained_bytes: 0,
                scripting_enabled: scripting,
                mode: DocumentMode::NoQuirks,
                encoding: encoding_rs::UTF_8,
                encoding_declaration: None,
                url: url::Url::parse("about:blank").expect("constant URL"),
                first_base: None,
                base_tracking: false,
                base_href_bytes: 0,
                details_groups: BTreeMap::new(),
                details_toggles: BTreeMap::new(),
                details_trackers: BTreeMap::new(),
                details_active: None,
                details_sequence: 0,
                details_name_bytes: 0,
                details_summaries: std::cell::RefCell::new(BTreeMap::new()),
            },
            stack: vec![],
            formatting: vec![],
            html: None,
            head: None,
            body: None,
            form: None,
            fragment_context: None,
            mode: InsertionMode::Initial,
            template_modes: vec![],
            scripting,
            frameset_ok: true,
            raw: None,
            raw_node: None,
            pending_table_text: String::new(),
            skip_lf: false,
            selected_contents: BTreeMap::new(),
            work: 50_000_000,
        }
    }
    fn current(&self) -> NodeId {
        self.stack.last().copied().unwrap_or(self.doc.root)
    }
    fn pop_open(&mut self) -> Option<NodeId> {
        let id = self.stack.pop()?;
        if self.html_tag(id) == Some("option") {
            self.complete_option(id);
        }
        Some(id)
    }
    fn truncate_open(&mut self, length: usize) {
        while self.stack.len() > length {
            self.pop_open();
        }
    }
    fn option_select(&mut self, option: NodeId) -> Option<NodeId> {
        let mut ancestor = self.doc.nodes[option].parent;
        let mut optgroup = false;
        while let Some(id) = ancestor {
            if !self.spend(1) {
                return None;
            }
            match self.html_tag(id) {
                Some("option" | "datalist" | "hr") => return None,
                Some("optgroup") if optgroup => return None,
                Some("optgroup") => optgroup = true,
                Some("select") => return Some(id),
                _ => {}
            }
            ancestor = self.doc.nodes[id].parent;
        }
        None
    }
    fn complete_option(&mut self, option: NodeId) {
        if let Some(select) = self.option_select(option)
            && let Some(Some(target)) = self.selected_contents.get(&select).copied()
            && self.initial_selected_option(select) == Some(option)
        {
            self.clone_option_contents(option, target);
        }
    }
    fn connect_selectedcontent(&mut self, id: NodeId) {
        let mut ancestor = self.doc.nodes[id].parent;
        let mut select = None;
        let mut disabled = false;
        let mut root = id;
        while let Some(parent) = ancestor {
            if !self.spend(1) {
                return;
            }
            match self.html_tag(parent) {
                Some("select") if select.is_none() => select = Some(parent),
                Some("select" | "option" | "selectedcontent") => disabled = true,
                _ => {}
            }
            root = parent;
            ancestor = self.doc.nodes[parent].parent;
        }
        // Template contents are not connected. No form-element connection
        // steps run until a later insertion connects them to a document.
        let Some(select) = select.filter(|_| root == self.doc.root) else {
            return;
        };
        if self.selected_contents.contains_key(&select) {
            return;
        }
        disabled |= self.doc.attr(select, "multiple").is_some();
        self.selected_contents
            .insert(select, (!disabled).then_some(id));
        if !disabled && let Some(option) = self.initial_selected_option(select) {
            self.clone_option_contents(option, id);
        }
    }
    fn initial_selected_option(&mut self, select: NodeId) -> Option<NodeId> {
        if self.doc.attr(select, "multiple").is_some() {
            return None;
        }
        let children = &self.doc.nodes[select].children;
        if children.len() > self.work {
            self.work = 0;
            return None;
        }
        self.work -= children.len();
        let mut pending = children
            .iter()
            .rev()
            .map(|&id| (id, false, false))
            .collect::<Vec<_>>();
        let mut first = None;
        let mut selected = None;
        while let Some((id, group, group_disabled)) = pending.pop() {
            if !self.spend(1) {
                return None;
            }
            let tag = self.html_tag(id).unwrap_or("");
            if tag == "option" {
                if self.doc.attr(id, "selected").is_some() {
                    selected = Some(id);
                }
                if first.is_none() && !group_disabled && self.doc.attr(id, "disabled").is_none() {
                    first = Some(id);
                }
                continue;
            }
            if matches!(tag, "select" | "hr" | "datalist") || tag == "optgroup" && group {
                continue;
            }
            let group_disabled =
                group_disabled || tag == "optgroup" && self.doc.attr(id, "disabled").is_some();
            let group = group || tag == "optgroup";
            let children = &self.doc.nodes[id].children;
            if children.len() > self.work {
                self.work = 0;
                return None;
            }
            self.work -= children.len();
            pending.extend(
                children
                    .iter()
                    .rev()
                    .map(|&child| (child, group, group_disabled)),
            );
        }
        let size_bytes = self.doc.attr(select, "size").map_or(0, str::len);
        if !self.spend(size_bytes) {
            return None;
        }
        let size = self
            .doc
            .attr(select, "size")
            .and_then(|value| {
                let value =
                    value.trim_start_matches(|ch: char| ch.is_ascii() && is_space(ch as u8));
                let value = value.strip_prefix('+').unwrap_or(value);
                let end = value.bytes().take_while(u8::is_ascii_digit).count();
                value[..end].parse::<u64>().ok()
            })
            .unwrap_or(1);
        selected.or_else(|| (size == 1).then_some(first).flatten())
    }
    /// Clone the selected option's actual children. Preflight includes hosted
    /// template fragments and charges bytes, nodes, traversal and destination
    /// depth before changing the target, so a quota failure is atomic.
    fn clone_option_contents(&mut self, option: NodeId, target: NodeId) {
        let mut depth = 0usize;
        let mut ancestor = Some(target);
        while let Some(id) = ancestor {
            if !self.spend(1) || depth >= MAX_DEPTH {
                return;
            }
            depth += 1;
            ancestor = self.doc.host_including_parent(id);
        }
        let base = self.doc.nodes.len();
        let roots = &self.doc.nodes[option].children;
        if roots.len() > self.work {
            self.work = 0;
            return;
        }
        self.work -= roots.len();
        let mut pending = roots
            .iter()
            .rev()
            .map(|&id| (id, Some(target), None, depth))
            .collect::<Vec<_>>();
        let mut plan = Vec::new();
        let mut bytes = 0usize;
        while let Some((source, parent, host, depth)) = pending.pop() {
            if depth > MAX_DEPTH || base + plan.len() >= MAX_NODES {
                return;
            }
            let node = &self.doc.nodes[source];
            let own_bytes = match &node.kind {
                NodeKind::Element(element) => element.retained_bytes(),
                NodeKind::Text(value) | NodeKind::Comment(value) => value.stored_bytes(),
                NodeKind::ProcessingInstruction { target, data } => {
                    target.len() + data.stored_bytes()
                }
                NodeKind::DocumentFragment { .. } => 0,
                _ => return,
            };
            let cost = own_bytes
                .saturating_add(node.children.len())
                .saturating_add(2);
            if cost > self.work {
                self.work = 0;
                return;
            }
            self.work -= cost;
            bytes += own_bytes;
            if bytes > MAX_DOM_BYTES.saturating_sub(self.doc.retained_bytes) {
                return;
            }
            let clone = base + plan.len();
            plan.push((source, parent, host));
            pending.extend(
                node.children
                    .iter()
                    .rev()
                    .map(|&child| (child, Some(clone), None, depth + 1)),
            );
            if let NodeKind::Element(element) = &node.kind
                && let Some(content) = element.template_contents
            {
                pending.push((content, None, Some(clone), depth + 1));
            }
        }
        if !self.spend(self.doc.nodes[target].children.len()) {
            return;
        }
        for child in std::mem::take(&mut self.doc.nodes[target].children) {
            self.doc.nodes[child].parent = None;
        }
        for (source, parent, host) in plan {
            let mut kind = self.doc.nodes[source].kind.clone();
            match &mut kind {
                NodeKind::Element(element) => element.template_contents = None,
                NodeKind::DocumentFragment { host: cloned_host } => *cloned_host = host,
                _ => {}
            }
            let id = self.doc.nodes.len();
            self.doc.nodes.push(Node {
                parent,
                children: vec![],
                kind,
            });
            if let Some(parent) = parent {
                self.doc.nodes[parent].children.push(id);
            }
            if let Some(host) = host
                && let NodeKind::Element(element) = &mut self.doc.nodes[host].kind
            {
                element.template_contents = Some(id);
            }
        }
        self.doc.retained_bytes += bytes;
    }
    fn adjusted_current(&self) -> NodeId {
        if self.stack.len() == 1 {
            self.fragment_context.unwrap_or_else(|| self.current())
        } else {
            self.current()
        }
    }
    fn current_tag(&self) -> &str {
        self.html_tag(self.current()).unwrap_or("")
    }
    fn html_tag(&self, id: NodeId) -> Option<&str> {
        (self.doc.namespace(id) == Some(Namespace::Html))
            .then(|| self.doc.tag(id))
            .flatten()
    }
    fn foreign(&self) -> bool {
        self.doc
            .namespace(self.adjusted_current())
            .is_some_and(|ns| ns != Namespace::Html)
    }
    fn math_text_integration(&self, id: NodeId) -> bool {
        self.doc.namespace(id) == Some(Namespace::MathMl)
            && matches!(self.doc.tag(id), Some("mi" | "mo" | "mn" | "ms" | "mtext"))
    }
    fn html_integration(&self, id: NodeId) -> bool {
        match self.doc.namespace(id) {
            Some(Namespace::Svg) => {
                matches!(self.doc.tag(id), Some("foreignObject" | "desc" | "title"))
            }
            Some(Namespace::MathMl) if self.doc.tag(id) == Some("annotation-xml") => {
                self.doc.attr(id, "encoding").is_some_and(|value| {
                    value.eq_ignore_ascii_case("text/html")
                        || value.eq_ignore_ascii_case("application/xhtml+xml")
                })
            }
            _ => false,
        }
    }
    fn foreign_characters(&self) -> bool {
        self.foreign()
            && !self.math_text_integration(self.adjusted_current())
            && !self.html_integration(self.adjusted_current())
    }
    fn dispatch_foreign(&self, token: &HtmlToken) -> bool {
        if !self.foreign() || matches!(token, HtmlToken::Eof) {
            return false;
        }
        if matches!(token, HtmlToken::Characters(_)) {
            return self.foreign_characters();
        }
        if token.is_start()
            && (self.math_text_integration(self.adjusted_current())
                && !matches!(token.tag(), "mglyph" | "malignmark")
                || self.html_integration(self.adjusted_current())
                || self.doc.namespace(self.adjusted_current()) == Some(Namespace::MathMl)
                    && self.doc.tag(self.adjusted_current()) == Some("annotation-xml")
                    && token.tag() == "svg")
        {
            return false;
        }
        true
    }
    fn foreign_scope_boundary(&self, id: NodeId) -> bool {
        match self.doc.namespace(id) {
            Some(Namespace::MathMl) => {
                self.math_text_integration(id) || self.doc.tag(id) == Some("annotation-xml")
            }
            Some(Namespace::Svg) => self.html_integration(id),
            _ => false,
        }
    }
    fn special_node(&self, id: NodeId) -> bool {
        self.html_tag(id).is_some_and(special_html) || self.foreign_scope_boundary(id)
    }
    /// Returns false when foreign recovery must use the current HTML insertion mode.
    fn in_foreign(&mut self, token: &HtmlToken) -> bool {
        match token {
            HtmlToken::Characters(text) => {
                if !text.bytes().all(|byte| byte == 0 || is_space(byte)) {
                    self.frameset_ok = false;
                }
                self.text(&text.replace('\0', "\u{fffd}"), false);
            }
            HtmlToken::Comment(text) => self.comment(text, None),
            HtmlToken::ProcessingInstruction { target, data } => {
                let id = self.doc.create_processing_instruction(target, data);
                self.doc.append_child(self.current(), id);
            }
            HtmlToken::Doctype(_) | HtmlToken::Eof => {}
            _ if foreign_breakout(token) => {
                while self
                    .doc
                    .namespace(self.current())
                    .is_some_and(|ns| ns != Namespace::Html)
                    && !self.math_text_integration(self.current())
                    && !self.html_integration(self.current())
                {
                    if !self.spend(1) {
                        return true;
                    }
                    self.pop_open();
                }
                return false;
            }
            HtmlToken::Start { self_closing, .. } => {
                if let Some(namespace) = self.doc.namespace(self.adjusted_current()) {
                    self.element_ns(token, namespace, false, !self_closing);
                }
            }
            HtmlToken::End(tag) => {
                for index in (1..self.stack.len()).rev() {
                    if !self.spend(1) {
                        return true;
                    }
                    let id = self.stack[index];
                    if self
                        .doc
                        .tag(id)
                        .is_some_and(|name| name.eq_ignore_ascii_case(tag))
                    {
                        self.truncate_open(index);
                        return true;
                    }
                    if self.doc.namespace(self.stack[index - 1]) == Some(Namespace::Html) {
                        return false;
                    }
                }
            }
        }
        true
    }
    fn scope(&self, tags: &[&str], scope: Scope) -> Option<usize> {
        let table = scope == Scope::Table;
        for (index, id) in self.stack.iter().enumerate().rev() {
            let tag = self.html_tag(*id).unwrap_or("");
            if tags.contains(&tag) {
                return Some(index);
            }
            if !table && self.foreign_scope_boundary(*id)
                || matches!(tag, "html" | "table" | "template")
                || !table
                    && matches!(
                        tag,
                        "applet" | "caption" | "td" | "th" | "marquee" | "object" | "select"
                    )
                || scope == Scope::Button && tag == "button"
                || scope == Scope::ListItem && matches!(tag, "ol" | "ul")
            {
                return None;
            }
        }
        None
    }
    fn clear_to(&mut self, tags: &[&str]) {
        while self.stack.len() > 1 && !tags.contains(&self.current_tag()) {
            self.pop_open();
        }
    }
    fn in_template(&self) -> bool {
        self.fragment_context
            .is_some_and(|id| self.html_tag(id) == Some("template"))
            || self
                .stack
                .iter()
                .any(|id| self.html_tag(*id) == Some("template"))
    }
    fn start_template(&mut self, token: &HtmlToken) {
        let id = self.element(token, false, true);
        if self.stack.last() == Some(&id) && self.doc.template_contents(id).is_some() {
            self.frameset_ok = false;
            self.template_modes.push(InsertionMode::InTemplate);
            self.mode = InsertionMode::InTemplate;
        }
    }
    fn close_template(&mut self) -> bool {
        let Some(index) = self
            .stack
            .iter()
            .rposition(|id| self.html_tag(*id) == Some("template"))
        else {
            return false;
        };
        // Thorough implied-end-tag generation and the subsequent pop have the
        // same final stack here; this parser does not expose parse-error counts.
        self.truncate_open(index);
        self.clear_formatting();
        self.template_modes.pop();
        self.reset_mode();
        true
    }
    fn close_in_scope(&mut self, tags: &[&str], scope: Scope) -> bool {
        if let Some(index) = self.scope(tags, scope) {
            let clear_formatting = self
                .html_tag(self.stack[index])
                .is_some_and(formatting_marker);
            self.truncate_open(index);
            if clear_formatting {
                self.clear_formatting();
            }
            true
        } else {
            false
        }
    }
    fn implied_end_tags(&mut self, except: Option<&str>) {
        while Some(self.current_tag()) != except
            && matches!(
                self.current_tag(),
                "dd" | "dt" | "li" | "optgroup" | "option" | "p" | "rb" | "rp" | "rt" | "rtc"
            )
        {
            if !self.spend(1) {
                return;
            }
            self.pop_open();
        }
    }
    fn close_list_item(&mut self, tags: &[&str]) {
        for index in (0..self.stack.len()).rev() {
            if !self.spend(1) {
                return;
            }
            let id = self.stack[index];
            let tag = self.html_tag(id).unwrap_or("");
            if tags.contains(&tag) {
                self.truncate_open(index);
                break;
            }
            if self.special_node(id) && !matches!(tag, "address" | "div" | "p") {
                break;
            }
        }
        self.close_in_scope(&["p"], Scope::Button);
    }
    fn reset_mode(&mut self) {
        self.mode = self
            .stack
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, id)| {
                match self.html_tag(if index == 0 {
                    self.fragment_context.unwrap_or(*id)
                } else {
                    *id
                }) {
                    Some("td" | "th") if index != 0 => Some(InsertionMode::InCell),
                    Some("tr") => Some(InsertionMode::InRow),
                    Some("tbody" | "thead" | "tfoot") => Some(InsertionMode::InTableBody),
                    Some("caption") => Some(InsertionMode::InCaption),
                    Some("colgroup") => Some(InsertionMode::InColumnGroup),
                    Some("table") => Some(InsertionMode::InTable),
                    Some("head") if index != 0 => Some(InsertionMode::InHead),
                    Some("body") => Some(InsertionMode::InBody),
                    Some("template") => self.template_modes.last().copied(),
                    Some("frameset") => Some(InsertionMode::InFrameset),
                    Some("html") => Some(if self.head.is_none() {
                        InsertionMode::BeforeHead
                    } else {
                        InsertionMode::AfterHead
                    }),
                    _ => None,
                }
            })
            .unwrap_or(InsertionMode::InBody);
    }
    fn location(&mut self, foster: bool) -> (NodeId, Option<usize>) {
        self.location_for(self.current(), foster)
    }
    fn location_for(&mut self, current: NodeId, foster: bool) -> (NodeId, Option<usize>) {
        if foster
            && matches!(
                self.html_tag(current),
                Some("table" | "tbody" | "tfoot" | "thead" | "tr")
            )
        {
            if let Some(index) = self
                .stack
                .iter()
                .rposition(|id| matches!(self.html_tag(*id), Some("table" | "template")))
            {
                let table = self.stack[index];
                if self.html_tag(table) == Some("template") {
                    return (self.doc.template_contents(table).unwrap_or(table), None);
                }
                if let Some(parent) = self.doc.nodes[table].parent {
                    let children = &self.doc.nodes[parent].children;
                    for (index, id) in children.iter().enumerate().rev() {
                        if self.work == 0 {
                            return (parent, None);
                        }
                        self.work -= 1;
                        if *id == table {
                            return (parent, Some(index));
                        }
                    }
                }
                let target = self.stack[index.saturating_sub(1)];
                return (self.doc.template_contents(target).unwrap_or(target), None);
            }
            return (self.html.unwrap_or(self.doc.root), None);
        }
        (self.doc.template_contents(current).unwrap_or(current), None)
    }
    fn attach(&mut self, node: NodeId, parent: NodeId, before: Option<usize>) {
        self.doc.append_child(parent, node);
        if self.doc.nodes.get(node).and_then(|n| n.parent) == Some(parent)
            && let Some(before) = before
        {
            let children = &mut self.doc.nodes[parent].children;
            if before < children.len().saturating_sub(1) {
                self.work = self.work.saturating_sub(children.len() - before);
                children.pop();
                children.insert(before, node);
            }
        }
    }
    fn element(&mut self, token: &HtmlToken, foster: bool, push: bool) -> NodeId {
        self.element_ns(token, Namespace::Html, foster, push)
    }
    fn element_ns(
        &mut self,
        token: &HtmlToken,
        namespace: Namespace,
        foster: bool,
        push: bool,
    ) -> NodeId {
        let HtmlToken::Start { tag, attrs, .. } = token else {
            return self.doc.root;
        };
        let tag = if namespace == Namespace::Svg {
            adjust_svg_tag(tag)
        } else {
            tag
        };
        if self.doc.nodes.len()
            >= MAX_NODES - usize::from(namespace == Namespace::Html && tag == "template")
            || tag.len() > MAX_DOM_BYTES.saturating_sub(self.doc.retained_bytes)
        {
            return self.doc.root;
        }
        // Tokenized HTML names need not satisfy createElement's author-facing name filter.
        self.doc.retained_bytes += tag.len();
        let id = self.doc.nodes.len();
        self.doc.nodes.push(Node {
            parent: None,
            children: vec![],
            kind: NodeKind::Element(Element {
                namespace,
                tag: tag.into(),
                attrs: BTreeMap::new(),
                attr_namespaces: BTreeMap::new(),
                template_contents: None,
            }),
        });
        self.doc.establish_template_contents(id);
        for (name, value) in attrs {
            let name = match namespace {
                Namespace::Html => name.as_str(),
                Namespace::Svg => adjust_svg_attribute(name),
                Namespace::MathMl if name == "definitionurl" => "definitionURL",
                Namespace::MathMl => name.as_str(),
            };
            if namespace != Namespace::Html
                && let Some(namespace) = AttributeNamespace::from_qualified_name(name)
            {
                self.doc.set_attr_ns(id, namespace, name, value);
            } else {
                self.doc.set_attr(id, name, value);
            }
        }
        let (parent, before) = self.location(foster);
        self.attach(id, parent, before);
        if namespace == Namespace::Html
            && tag == "selectedcontent"
            && self.doc.nodes[id].parent.is_some()
        {
            self.connect_selectedcontent(id);
        }
        if namespace == Namespace::Html
            && tag == "meta"
            && self.fragment_context.is_none()
            && self.doc.encoding_declaration.is_none()
            && self.doc.nodes[id].parent.is_some()
        {
            self.doc.encoding_declaration = crate::text_encoding::meta_encoding(
                self.doc.attr(id, "charset"),
                self.doc.attr(id, "http-equiv"),
                self.doc.attr(id, "content"),
            );
        }
        if push && self.stack.len() < MAX_DEPTH - 3 && self.doc.nodes[id].parent.is_some() {
            self.stack.push(id);
            if namespace == Namespace::Html && formatting_marker(tag) {
                self.formatting.push(None);
            }
        }
        id
    }
    fn spend(&mut self, amount: usize) -> bool {
        if amount > self.work {
            self.work = 0;
            false
        } else {
            self.work -= amount;
            true
        }
    }
    fn clear_formatting(&mut self) {
        while let Some(entry) = self.formatting.pop() {
            if !self.spend(1) || entry.is_none() {
                break;
            }
        }
    }
    fn formatting_index(&mut self, subject: &str) -> Option<usize> {
        for index in (0..self.formatting.len()).rev() {
            if !self.spend(1) {
                return None;
            }
            let id = self.formatting[index]?;
            if self.html_tag(id) == Some(subject) {
                return Some(index);
            }
        }
        None
    }
    fn node_in_scope(&self, target: NodeId) -> bool {
        for id in self.stack.iter().rev() {
            if *id == target {
                return true;
            }
            if self.foreign_scope_boundary(*id)
                || self.html_tag(*id).is_some_and(|tag| {
                    matches!(
                        tag,
                        "applet"
                            | "caption"
                            | "html"
                            | "table"
                            | "td"
                            | "th"
                            | "marquee"
                            | "object"
                            | "select"
                            | "template"
                    )
                })
            {
                return false;
            }
        }
        false
    }
    fn push_formatting(&mut self, id: NodeId) {
        if id == self.doc.root || !self.stack.contains(&id) {
            return;
        }
        let mut identical = Vec::new();
        for index in (0..self.formatting.len()).rev() {
            let Some(other) = self.formatting[index] else {
                break;
            };
            if !self.spend(1) {
                return;
            }
            let (NodeKind::Element(element), NodeKind::Element(candidate)) =
                (&self.doc.nodes[id].kind, &self.doc.nodes[other].kind)
            else {
                continue;
            };
            if element.tag != candidate.tag || element.attrs.len() != candidate.attrs.len() {
                continue;
            }
            let cost = element.tag.len()
                + element
                    .attrs
                    .iter()
                    .map(|(name, value)| name.len() + value.len() + 1)
                    .sum::<usize>();
            if cost > self.work {
                self.work = 0;
                return;
            }
            self.work -= cost;
            if element.attrs == candidate.attrs {
                identical.push(index);
            }
        }
        // The Noah's Ark rule keeps at most three identical entries after a marker.
        if identical.len() >= 3 {
            if !self.spend(self.formatting.len()) {
                return;
            }
            self.formatting.remove(*identical.last().unwrap());
        }
        if self.formatting.len() < MAX_NODES {
            self.formatting.push(Some(id));
        }
    }
    fn clone_formatting(&mut self, original: NodeId) -> Option<NodeId> {
        let NodeKind::Element(element) = &self.doc.nodes.get(original)?.kind else {
            return None;
        };
        let bytes = element.retained_bytes();
        let cost = bytes.saturating_add(element.attrs.len()).saturating_add(1);
        if cost > self.work
            || bytes > MAX_DOM_BYTES.saturating_sub(self.doc.retained_bytes)
            || self.doc.nodes.len() >= MAX_NODES
        {
            self.work = 0;
            return None;
        }
        self.work -= cost;
        let kind = NodeKind::Element(element.clone());
        let id = self.doc.nodes.len();
        self.doc.nodes.push(Node {
            parent: None,
            children: vec![],
            kind,
        });
        self.doc.retained_bytes += bytes;
        Some(id)
    }
    fn reconstruct_formatting(&mut self, foster: bool) {
        let mut first = self.formatting.len();
        while first > 0 {
            if !self.spend(self.stack.len() + 1) {
                return;
            }
            let Some(id) = self.formatting[first - 1] else {
                break;
            };
            if self.stack.contains(&id) {
                break;
            }
            first -= 1;
        }
        for index in first..self.formatting.len() {
            if self.stack.len() >= MAX_DEPTH - 3 {
                self.work = 0;
                return;
            }
            let Some(original) = self.formatting[index] else {
                continue;
            };
            let Some(id) = self.clone_formatting(original) else {
                return;
            };
            let (parent, before) = self.location(foster);
            if !self.reparent(id, parent, before) {
                return;
            }
            self.stack.push(id);
            self.formatting[index] = Some(id);
        }
    }
    // Reparenting can revisit large subtrees. Charge every visited node and every
    // sibling-vector operation before invoking the DOM's cycle/depth checks.
    fn reparent(&mut self, node: NodeId, parent: NodeId, before: Option<usize>) -> bool {
        if node == parent
            || node == self.doc.root
            || node >= self.doc.nodes.len()
            || parent >= self.doc.nodes.len()
        {
            self.work = 0;
            return false;
        }
        let reference =
            before.and_then(|index| self.doc.nodes[parent].children.get(index).copied());
        if reference == Some(node) {
            return true;
        }
        let mut cursor = Some(parent);
        let mut ancestors = 0;
        while let Some(id) = cursor {
            if !self.spend(1) || id == node || ancestors >= MAX_DEPTH {
                self.work = 0;
                return false;
            }
            ancestors += 1;
            cursor = self.doc.nodes[id].parent;
        }
        let mut pending = vec![(node, 1usize)];
        let mut visited = 0;
        while let Some((id, depth)) = pending.pop() {
            visited += 1;
            if !self.spend(1) || visited > MAX_NODES || ancestors + depth > MAX_DEPTH {
                self.work = 0;
                return false;
            }
            pending.extend(
                self.doc.nodes[id]
                    .children
                    .iter()
                    .map(|child| (*child, depth + 1)),
            );
        }
        let old_cost = self.doc.nodes[node]
            .parent
            .map_or(0, |id| self.doc.nodes[id].children.len());
        let new_cost = if before.is_some() {
            self.doc.nodes[parent].children.len().saturating_mul(2)
        } else {
            1
        };
        if !self.spend(old_cost.saturating_add(new_cost)) {
            return false;
        }
        self.doc.append_child(parent, node);
        if self.doc.nodes[node].parent != Some(parent) {
            self.work = 0;
            return false;
        }
        if let Some(reference) = reference {
            let children = &mut self.doc.nodes[parent].children;
            if let Some(index) = children.iter().position(|id| *id == reference) {
                children.pop();
                children.insert(index, node);
            }
        }
        true
    }
    fn wrap_children(&mut self, parent: NodeId, wrapper: NodeId) -> bool {
        if self.doc.nodes[wrapper].parent.is_some() || !self.doc.nodes[wrapper].children.is_empty()
        {
            self.work = 0;
            return false;
        }
        let mut ancestors = 1usize; // The wrapper adds one level to every child.
        let mut cursor = Some(parent);
        while let Some(id) = cursor {
            if !self.spend(1) || ancestors >= MAX_DEPTH {
                self.work = 0;
                return false;
            }
            ancestors += 1;
            cursor = self.doc.nodes[id].parent;
        }
        if !self.spend(self.doc.nodes[parent].children.len()) {
            return false;
        }
        let mut pending: Vec<_> = self.doc.nodes[parent]
            .children
            .iter()
            .map(|id| (*id, 1usize))
            .collect();
        let mut visited = 0;
        while let Some((id, depth)) = pending.pop() {
            visited += 1;
            if !self.spend(1) || visited > MAX_NODES || ancestors + depth > MAX_DEPTH {
                self.work = 0;
                return false;
            }
            pending.extend(
                self.doc.nodes[id]
                    .children
                    .iter()
                    .map(|child| (*child, depth + 1)),
            );
        }
        // Commit in linear time only after the complete move has passed its checks.
        let children = std::mem::take(&mut self.doc.nodes[parent].children);
        for child in &children {
            self.doc.nodes[*child].parent = Some(wrapper);
        }
        self.doc.nodes[wrapper].children = children;
        self.doc.nodes[wrapper].parent = Some(parent);
        self.doc.nodes[parent].children.push(wrapper);
        true
    }
    fn generic_end(&mut self, subject: &str) {
        for index in (0..self.stack.len()).rev() {
            let current = self.html_tag(self.stack[index]).unwrap_or("");
            if current == subject {
                self.truncate_open(index);
                return;
            }
            if self.special_node(self.stack[index]) {
                return;
            }
        }
    }
    fn adoption_agency(&mut self, subject: &str, foster: bool) {
        if !self.spend(self.formatting.len() + self.stack.len()) {
            return;
        }
        if self.current_tag() == subject && !self.formatting.contains(&Some(self.current())) {
            self.pop_open();
            return;
        }
        for _ in 0..8 {
            if !self.spend(self.stack.len() + self.formatting.len() + 1) {
                return;
            }
            let Some(format_index) = self.formatting_index(subject) else {
                if self.work > 0 {
                    self.generic_end(subject);
                }
                return;
            };
            let Some(format) = self.formatting[format_index] else {
                return;
            };
            let Some(stack_index) = self.stack.iter().position(|id| *id == format) else {
                self.formatting.remove(format_index);
                return;
            };
            if !self.node_in_scope(format) {
                return;
            }
            let Some(block_index) = (stack_index + 1..self.stack.len())
                .find(|index| self.special_node(self.stack[*index]))
            else {
                self.truncate_open(stack_index);
                self.formatting.remove(format_index);
                return;
            };
            let Some(common_ancestor) = stack_index.checked_sub(1).map(|index| self.stack[index])
            else {
                self.work = 0;
                return;
            };
            let furthest_block = self.stack[block_index];
            let mut bookmark = format_index;
            let mut last_node = furthest_block;
            let mut cursor = block_index;
            let mut inner = 0usize;
            loop {
                if !self.spend(self.formatting.len() + self.stack.len() + 1) {
                    return;
                }
                inner += 1;
                if cursor == 0 {
                    self.work = 0;
                    return;
                }
                cursor -= 1;
                let node = self.stack[cursor];
                if node == format {
                    break;
                }
                let mut active_index = self
                    .formatting
                    .iter()
                    .position(|entry| *entry == Some(node));
                if inner > 3
                    && let Some(index) = active_index
                {
                    self.formatting.remove(index);
                    if index < bookmark {
                        bookmark -= 1;
                    }
                    active_index = None;
                }
                let Some(index) = active_index else {
                    self.stack.remove(cursor);
                    continue;
                };
                let Some(new_node) = self.clone_formatting(node) else {
                    return;
                };
                self.formatting[index] = Some(new_node);
                self.stack[cursor] = new_node;
                if last_node == furthest_block {
                    bookmark = index + 1;
                }
                if !self.reparent(last_node, new_node, None) {
                    return;
                }
                last_node = new_node;
            }
            let (parent, before) = self.location_for(common_ancestor, foster);
            if !self.reparent(last_node, parent, before) {
                return;
            }
            let Some(new_format) = self.clone_formatting(format) else {
                return;
            };
            if !self.wrap_children(furthest_block, new_format) {
                return;
            }
            if !self.spend(self.formatting.len() + self.stack.len()) {
                return;
            }
            if let Some(index) = self
                .formatting
                .iter()
                .position(|entry| *entry == Some(format))
            {
                self.formatting.remove(index);
                if index < bookmark {
                    bookmark -= 1;
                }
            }
            self.formatting
                .insert(bookmark.min(self.formatting.len()), Some(new_format));
            if let Some(index) = self.stack.iter().position(|id| *id == format) {
                self.stack.remove(index);
            }
            if let Some(index) = self.stack.iter().position(|id| *id == furthest_block) {
                self.stack.insert(index + 1, new_format);
            }
        }
    }
    fn merge_attrs(&mut self, id: Option<NodeId>, token: &HtmlToken) {
        if self
            .stack
            .iter()
            .any(|id| self.html_tag(*id) == Some("template"))
        {
            return;
        }
        if let (Some(id), HtmlToken::Start { attrs, .. }) = (id, token) {
            for (name, value) in attrs {
                if self.doc.attr(id, name).is_none() {
                    self.doc.set_attr(id, name, value);
                }
            }
        }
    }
    fn text(&mut self, text: &str, foster: bool) {
        if text.is_empty() {
            return;
        }
        let (parent, before) = self.location(foster);
        if let Some(index) = before {
            let previous = index
                .checked_sub(1)
                .and_then(|index| self.doc.nodes[parent].children.get(index))
                .copied();
            if let Some(previous) = previous
                && let NodeKind::Text(value) = &mut self.doc.nodes[previous].kind
                && let Some(value) = value.scalar_mut()
            {
                let available = MAX_DOM_BYTES.saturating_sub(self.doc.retained_bytes);
                let text = &text[..floor_boundary(text, text.len().min(available))];
                value.push_str(text);
                self.doc.retained_bytes += text.len();
            } else {
                let node = self.doc.create_text_node(text);
                self.attach(node, parent, before);
            }
        } else {
            self.doc.push_text(parent, text.into());
        }
    }
    fn comment(&mut self, text: &str, parent: Option<NodeId>) {
        let id = self.doc.create_comment(text);
        let (parent, before) = parent
            .map(|parent| (parent, None))
            .unwrap_or_else(|| self.location(false));
        self.attach(id, parent, before);
    }
    fn raw_element(&mut self, token: &HtmlToken, foster: bool) {
        let id = self.element(token, foster, true);
        self.raw_node = Some((id, self.mode));
        self.raw = Some((
            token.tag().into(),
            matches!(token.tag(), "title" | "textarea"),
        ));
        self.skip_lf = token.tag() == "textarea";
    }
    fn process(&mut self, mut token: HtmlToken) {
        use InsertionMode::*;
        if !matches!(token, HtmlToken::Characters(_)) && !self.pending_table_text.is_empty() {
            let text = std::mem::take(&mut self.pending_table_text);
            let foster = !text.bytes().all(is_space);
            if foster {
                self.reconstruct_formatting(true);
            }
            if self.work == 0 {
                return;
            }
            self.text(&text, foster);
        }
        // Every reprocessing transition consumes budget; malformed input cannot spin indefinitely.
        for _ in 0..MAX_DEPTH + 32 {
            let cost = self.stack.len() + 1;
            if self.work < cost || self.doc.nodes.len() >= MAX_NODES {
                self.work = 0;
                return;
            }
            self.work -= cost;
            if let Some((node, original_mode)) = self.raw_node {
                match &token {
                    HtmlToken::Characters(text) => {
                        self.doc.push_text(node, text.clone());
                        return;
                    }
                    HtmlToken::End(tag) if self.html_tag(node) == Some(tag) => {
                        if self.stack.last() == Some(&node) {
                            self.pop_open();
                        }
                        self.raw_node = None;
                        self.mode = original_mode;
                        return;
                    }
                    HtmlToken::Eof => {
                        if self.stack.last() == Some(&node) {
                            self.pop_open();
                        }
                        self.raw_node = None;
                        self.mode = original_mode;
                        continue;
                    }
                    _ => {}
                }
            }
            if self.dispatch_foreign(&token) && self.in_foreign(&token) {
                return;
            }
            if matches!(token, HtmlToken::Eof) && self.close_template() {
                continue;
            }
            if let HtmlToken::ProcessingInstruction { target, data } = &token {
                let parent = match self.mode {
                    Initial | BeforeHtml | AfterAfterBody | AfterAfterFrameset => self.doc.root,
                    AfterBody => self.html.unwrap_or(self.doc.root),
                    _ => self.location(false).0,
                };
                let id = self.doc.create_processing_instruction(target, data);
                self.doc.append_child(parent, id);
                return;
            }
            if let HtmlToken::Characters(text) = &mut token {
                if matches!(self.mode, InTable | InTableBody | InRow)
                    && matches!(
                        self.current_tag(),
                        "table" | "tbody" | "template" | "tfoot" | "thead" | "tr"
                    )
                {
                    let available = MAX_DOM_BYTES
                        .saturating_sub(self.doc.retained_bytes)
                        .saturating_sub(self.pending_table_text.len());
                    self.pending_table_text
                        .push_str(&text[..floor_boundary(text, text.len().min(available))]);
                    return;
                }
                if matches!(
                    self.current_tag(),
                    "script"
                        | "style"
                        | "title"
                        | "textarea"
                        | "xmp"
                        | "iframe"
                        | "noembed"
                        | "noframes"
                ) || self.scripting && self.current_tag() == "noscript"
                {
                    self.text(text, false);
                    return;
                }
                if matches!(
                    self.mode,
                    Initial
                        | BeforeHtml
                        | BeforeHead
                        | InHead
                        | InHeadNoscript
                        | AfterHead
                        | InColumnGroup
                ) {
                    let leading = text.bytes().take_while(|b| is_space(*b)).count();
                    if leading > 0 {
                        if !matches!(self.mode, Initial | BeforeHtml | BeforeHead) {
                            self.text(&text[..leading], false);
                        }
                        text.drain(..leading);
                    }
                    if text.is_empty() {
                        return;
                    }
                }
            }
            let tag = token.tag();
            let start = token.is_start();
            let end = token.is_end();
            // Raw text consumes its end tag before ordinary insertion-mode dispatch.
            if end
                && tag == self.current_tag()
                && matches!(
                    tag,
                    "script"
                        | "style"
                        | "title"
                        | "textarea"
                        | "xmp"
                        | "iframe"
                        | "noembed"
                        | "noframes"
                )
            {
                self.pop_open();
                return;
            }
            match self.mode {
                Initial => match &token {
                    HtmlToken::Characters(_) if token.whitespace() => return,
                    HtmlToken::Comment(text) => {
                        self.comment(text, Some(self.doc.root));
                        return;
                    }
                    HtmlToken::Doctype(doctype) => {
                        self.doc.mode = doctype_mode(doctype);
                        let id = self.doc.create_doctype(doctype.clone());
                        self.doc.append_child(self.doc.root, id);
                        self.mode = BeforeHtml;
                        return;
                    }
                    _ => {
                        self.doc.mode = DocumentMode::Quirks;
                        self.mode = BeforeHtml;
                    }
                },
                BeforeHtml => match &token {
                    HtmlToken::Doctype(_) => return,
                    HtmlToken::Characters(_) if token.whitespace() => return,
                    HtmlToken::Comment(text) => {
                        self.comment(text, Some(self.doc.root));
                        return;
                    }
                    _ if end && !matches!(tag, "head" | "body" | "html" | "br") => return,
                    _ => {
                        let id = if start && tag == "html" {
                            self.element(&token, false, true)
                        } else {
                            self.element(&HtmlToken::start("html"), false, true)
                        };
                        self.html = Some(id);
                        self.mode = BeforeHead;
                        if start && tag == "html" {
                            return;
                        }
                    }
                },
                BeforeHead => match &token {
                    HtmlToken::Doctype(_) => return,
                    HtmlToken::Characters(_) if token.whitespace() => return,
                    HtmlToken::Comment(text) => {
                        self.comment(text, None);
                        return;
                    }
                    _ if start && tag == "html" => {
                        self.merge_attrs(self.html, &token);
                        return;
                    }
                    _ if end && !matches!(tag, "head" | "body" | "html" | "br") => return,
                    _ => {
                        let id = if start && tag == "head" {
                            self.element(&token, false, true)
                        } else {
                            self.element(&HtmlToken::start("head"), false, true)
                        };
                        self.head = Some(id);
                        self.mode = InHead;
                        if start && tag == "head" {
                            return;
                        }
                    }
                },
                InHead => match &token {
                    HtmlToken::Characters(text)
                        if token.whitespace()
                            || matches!(
                                self.current_tag(),
                                "script" | "style" | "title" | "noframes" | "noscript"
                            ) =>
                    {
                        self.text(text, false);
                        return;
                    }
                    HtmlToken::Comment(text) => {
                        self.comment(text, None);
                        return;
                    }
                    HtmlToken::Doctype(_) => return,
                    _ if start && tag == "html" => {
                        self.merge_attrs(self.html, &token);
                        return;
                    }
                    _ if start
                        && matches!(tag, "base" | "basefont" | "bgsound" | "link" | "meta") =>
                    {
                        self.element(&token, false, false);
                        return;
                    }
                    _ if start && matches!(tag, "title" | "style" | "noframes" | "script") => {
                        self.raw_element(&token, false);
                        return;
                    }
                    _ if start && tag == "noscript" => {
                        if self.scripting {
                            self.raw_element(&token, false);
                        } else {
                            self.element(&token, false, true);
                            self.mode = InHeadNoscript;
                        }
                        return;
                    }
                    _ if end
                        && tag == "noscript"
                        && self.scripting
                        && self.current_tag() == "noscript" =>
                    {
                        self.pop_open();
                        return;
                    }
                    _ if start && tag == "template" => {
                        self.start_template(&token);
                        return;
                    }
                    _ if end && tag == "template" => {
                        self.close_template();
                        return;
                    }
                    _ if start && tag == "head"
                        || end && !matches!(tag, "head" | "body" | "html" | "br" | "template") =>
                    {
                        return;
                    }
                    _ => {
                        self.clear_to(&["head", "html"]);
                        if self.current_tag() == "head" {
                            self.pop_open();
                        }
                        self.mode = AfterHead;
                        if end && tag == "head" {
                            return;
                        }
                    }
                },
                InHeadNoscript => match &token {
                    _ if end && tag == "noscript" => {
                        self.pop_open();
                        self.mode = InHead;
                        return;
                    }
                    _ if start && matches!(tag, "head" | "noscript") || end && tag != "br" => {
                        return;
                    }
                    HtmlToken::Doctype(_) => return,
                    HtmlToken::Comment(text) => {
                        self.comment(text, None);
                        return;
                    }
                    HtmlToken::Characters(text) if token.whitespace() => {
                        self.text(text, false);
                        return;
                    }
                    _ if start && matches!(tag, "basefont" | "bgsound" | "link" | "meta") => {
                        self.element(&token, false, false);
                        return;
                    }
                    _ if start && matches!(tag, "style" | "noframes") => {
                        self.raw_element(&token, false);
                        return;
                    }
                    _ if start && tag == "html" => {
                        self.merge_attrs(self.html, &token);
                        return;
                    }
                    _ => {
                        self.pop_open();
                        self.mode = InHead;
                    }
                },
                AfterHead => match &token {
                    HtmlToken::Characters(text) if token.whitespace() => {
                        self.text(text, false);
                        return;
                    }
                    HtmlToken::Comment(text) => {
                        self.comment(text, None);
                        return;
                    }
                    HtmlToken::Doctype(_) => return,
                    _ if end && tag == "template" => {
                        self.close_template();
                        return;
                    }
                    _ if start && tag == "html" => {
                        self.merge_attrs(self.html, &token);
                        return;
                    }
                    _ if start && tag == "frameset" => {
                        self.element(&token, false, true);
                        self.mode = InFrameset;
                        return;
                    }
                    _ if start
                        && matches!(
                            tag,
                            "base"
                                | "basefont"
                                | "bgsound"
                                | "link"
                                | "meta"
                                | "noframes"
                                | "script"
                                | "style"
                                | "template"
                                | "title"
                        ) =>
                    {
                        if let Some(head) = self.head {
                            self.stack.push(head);
                            self.in_body(&token, false);
                            if let Some(index) = self.stack.iter().position(|id| *id == head) {
                                self.stack.remove(index);
                            }
                        }
                        return;
                    }
                    _ if start && tag == "head"
                        || end && !matches!(tag, "body" | "html" | "br" | "template") =>
                    {
                        return;
                    }
                    _ => {
                        let id = if start && tag == "body" {
                            self.element(&token, false, true)
                        } else {
                            self.element(&HtmlToken::start("body"), false, true)
                        };
                        self.body = Some(id);
                        self.frameset_ok = !(start && tag == "body");
                        self.mode = InBody;
                        if start && tag == "body" {
                            return;
                        }
                    }
                },
                InTable => {
                    if self.in_table(&token) {
                        continue;
                    }
                    return;
                }
                InCaption => {
                    if end && tag == "caption"
                        || start
                            && matches!(
                                tag,
                                "caption"
                                    | "col"
                                    | "colgroup"
                                    | "tbody"
                                    | "td"
                                    | "tfoot"
                                    | "th"
                                    | "thead"
                                    | "tr"
                            )
                        || end && tag == "table"
                    {
                        if !self.close_in_scope(&["caption"], Scope::Table) {
                            return;
                        }
                        self.mode = InTable;
                        if end && tag == "caption" {
                            return;
                        }
                    } else if end
                        && matches!(
                            tag,
                            "body"
                                | "col"
                                | "colgroup"
                                | "html"
                                | "tbody"
                                | "td"
                                | "tfoot"
                                | "th"
                                | "thead"
                                | "tr"
                        )
                    {
                        return;
                    } else {
                        self.in_body(&token, false);
                        return;
                    }
                }
                InColumnGroup => match &token {
                    HtmlToken::Characters(text) if token.whitespace() => {
                        self.text(text, false);
                        return;
                    }
                    HtmlToken::Comment(text) => {
                        self.comment(text, None);
                        return;
                    }
                    HtmlToken::Doctype(_) => return,
                    _ if start && tag == "html" => {
                        self.merge_attrs(self.html, &token);
                        return;
                    }
                    _ if start && tag == "col" => {
                        self.element(&token, false, false);
                        return;
                    }
                    _ if end && tag == "col" => return,
                    _ if tag == "template" => {
                        self.in_body(&token, false);
                        return;
                    }
                    _ => {
                        if self.current_tag() != "colgroup" {
                            return;
                        }
                        self.pop_open();
                        self.mode = InTable;
                        if end && tag == "colgroup" {
                            return;
                        }
                    }
                },
                InTableBody => {
                    if start && tag == "tr" {
                        self.clear_to(&["tbody", "tfoot", "thead", "template", "html"]);
                        self.element(&token, false, true);
                        self.mode = InRow;
                        return;
                    }
                    if start && matches!(tag, "td" | "th") {
                        self.clear_to(&["tbody", "tfoot", "thead", "template", "html"]);
                        self.element(&HtmlToken::start("tr"), false, true);
                        self.mode = InRow;
                        continue;
                    }
                    if end && matches!(tag, "tbody" | "tfoot" | "thead") {
                        if self.scope(&[tag], Scope::Table).is_some() {
                            self.clear_to(&["tbody", "tfoot", "thead", "template", "html"]);
                            self.pop_open();
                            self.mode = InTable;
                        }
                        return;
                    }
                    if start
                        && matches!(
                            tag,
                            "caption" | "col" | "colgroup" | "tbody" | "tfoot" | "thead"
                        )
                        || end && tag == "table"
                    {
                        if self
                            .scope(&["tbody", "thead", "tfoot"], Scope::Table)
                            .is_none()
                        {
                            return;
                        }
                        self.clear_to(&["tbody", "tfoot", "thead", "template", "html"]);
                        self.pop_open();
                        self.mode = InTable;
                        continue;
                    }
                    if end
                        && matches!(
                            tag,
                            "body" | "caption" | "col" | "colgroup" | "html" | "td" | "th" | "tr"
                        )
                    {
                        return;
                    }
                    if self.in_table(&token) {
                        continue;
                    }
                    return;
                }
                InRow => {
                    if start && matches!(tag, "td" | "th") {
                        self.clear_to(&["tr", "template", "html"]);
                        self.element(&token, false, true);
                        self.mode = InCell;
                        return;
                    }
                    if end && tag == "tr" {
                        if self.close_in_scope(&["tr"], Scope::Table) {
                            self.mode = InTableBody;
                        }
                        return;
                    }
                    if start
                        && matches!(
                            tag,
                            "caption" | "col" | "colgroup" | "tbody" | "tfoot" | "thead" | "tr"
                        )
                        || end && matches!(tag, "table" | "tbody" | "tfoot" | "thead")
                    {
                        if end && tag != "table" && self.scope(&[tag], Scope::Table).is_none() {
                            return;
                        }
                        if !self.close_in_scope(&["tr"], Scope::Table) {
                            return;
                        }
                        self.mode = InTableBody;
                        continue;
                    }
                    if end
                        && matches!(
                            tag,
                            "body" | "caption" | "col" | "colgroup" | "html" | "td" | "th"
                        )
                    {
                        return;
                    }
                    if self.in_table(&token) {
                        continue;
                    }
                    return;
                }
                InCell => {
                    if end && matches!(tag, "td" | "th") {
                        if self.close_in_scope(&[tag], Scope::Table) {
                            self.mode = InRow;
                        }
                        return;
                    }
                    if start
                        && matches!(
                            tag,
                            "caption"
                                | "col"
                                | "colgroup"
                                | "tbody"
                                | "td"
                                | "tfoot"
                                | "th"
                                | "thead"
                                | "tr"
                        )
                        || end && matches!(tag, "table" | "tbody" | "tfoot" | "thead" | "tr")
                    {
                        if end && self.scope(&[tag], Scope::Table).is_none() {
                            return;
                        }
                        if !self.close_in_scope(&["td", "th"], Scope::Table) {
                            return;
                        }
                        self.mode = InRow;
                        continue;
                    }
                    if end && matches!(tag, "body" | "caption" | "col" | "colgroup" | "html") {
                        return;
                    }
                    self.in_body(&token, false);
                    return;
                }
                AfterBody => match &token {
                    HtmlToken::Characters(_) if token.whitespace() => {
                        self.in_body(&token, false);
                        return;
                    }
                    HtmlToken::Comment(text) => {
                        self.comment(text, self.html);
                        return;
                    }
                    HtmlToken::Doctype(_) | HtmlToken::Eof => return,
                    _ if start && tag == "html" => {
                        self.merge_attrs(self.html, &token);
                        return;
                    }
                    _ if end && tag == "html" => {
                        if self.fragment_context.is_none() {
                            self.mode = AfterAfterBody;
                        }
                        return;
                    }
                    _ => {
                        self.mode = InBody;
                    }
                },
                AfterAfterBody => match &token {
                    HtmlToken::Comment(text) => {
                        self.comment(text, Some(self.doc.root));
                        return;
                    }
                    HtmlToken::Doctype(_) | HtmlToken::Eof => return,
                    HtmlToken::Characters(_) if token.whitespace() => {
                        self.in_body(&token, false);
                        return;
                    }
                    _ if start && tag == "html" => {
                        self.merge_attrs(self.html, &token);
                        return;
                    }
                    _ => {
                        self.mode = InBody;
                    }
                },
                InTemplate => match &token {
                    HtmlToken::Characters(_) | HtmlToken::Comment(_) | HtmlToken::Doctype(_) => {
                        self.in_body(&token, false);
                        return;
                    }
                    _ if start
                        && matches!(
                            tag,
                            "base"
                                | "basefont"
                                | "bgsound"
                                | "link"
                                | "meta"
                                | "noframes"
                                | "script"
                                | "style"
                                | "template"
                                | "title"
                        )
                        || end && tag == "template" =>
                    {
                        self.in_body(&token, false);
                        return;
                    }
                    HtmlToken::Start { .. } => {
                        let mode = match tag {
                            "caption" | "colgroup" | "tbody" | "tfoot" | "thead" => InTable,
                            "col" => InColumnGroup,
                            "tr" => InTableBody,
                            "td" | "th" => InRow,
                            _ => InBody,
                        };
                        if let Some(current) = self.template_modes.last_mut() {
                            *current = mode;
                        }
                        self.mode = mode;
                    }
                    _ => return,
                },
                InBody => {
                    self.in_body(&token, false);
                    return;
                }
                AfterFrameset | AfterAfterFrameset => {
                    match &token {
                        HtmlToken::Characters(text) => {
                            let spaces = text
                                .chars()
                                .filter(|ch| ch.is_ascii() && is_space(*ch as u8))
                                .collect::<String>();
                            self.text(&spaces, false);
                        }
                        HtmlToken::Comment(text) => {
                            self.comment(
                                text,
                                (self.mode == AfterAfterFrameset).then_some(self.doc.root),
                            );
                        }
                        _ if start && tag == "html" => self.merge_attrs(self.html, &token),
                        _ if start && tag == "noframes" => self.raw_element(&token, false),
                        _ if end && tag == "html" && self.mode == AfterFrameset => {
                            self.mode = AfterAfterFrameset
                        }
                        _ => {}
                    }
                    return;
                }
                InFrameset => {
                    match &token {
                        HtmlToken::Characters(text) => {
                            let spaces = text
                                .chars()
                                .filter(|ch| ch.is_ascii() && is_space(*ch as u8))
                                .collect::<String>();
                            self.text(&spaces, false);
                        }
                        HtmlToken::Comment(text) => self.comment(text, None),
                        _ if start && tag == "html" => self.merge_attrs(self.html, &token),
                        _ if start && tag == "frameset" => {
                            self.element(&token, false, true);
                        }
                        _ if start && tag == "frame" => {
                            self.element(&token, false, false);
                        }
                        _ if start && tag == "noframes" => self.raw_element(&token, false),
                        _ if end && tag == "frameset" && self.current_tag() != "html" => {
                            self.pop_open();
                            if self.fragment_context.is_none() && self.current_tag() != "frameset" {
                                self.mode = AfterFrameset;
                            }
                        }
                        _ => {}
                    }
                    return;
                }
            }
        }
    }
    // Returns true only when the same token must be reprocessed in a changed mode.
    fn in_table(&mut self, token: &HtmlToken) -> bool {
        use InsertionMode::*;
        let tag = token.tag();
        let start = token.is_start();
        let end = token.is_end();
        match token {
            HtmlToken::Characters(text)
                if matches!(
                    self.current_tag(),
                    "table" | "tbody" | "template" | "tfoot" | "thead" | "tr"
                ) =>
            {
                self.text(text, !token.whitespace());
            }
            HtmlToken::Comment(text) => self.comment(text, None),
            HtmlToken::Doctype(_) | HtmlToken::Eof | HtmlToken::ProcessingInstruction { .. } => {}
            _ if start && matches!(tag, "caption" | "colgroup" | "tbody" | "tfoot" | "thead") => {
                self.clear_to(&["table", "template", "html"]);
                self.element(token, false, true);
                self.mode = match tag {
                    "caption" => InCaption,
                    "colgroup" => InColumnGroup,
                    _ => InTableBody,
                };
            }
            _ if start && tag == "col" => {
                self.clear_to(&["table", "template", "html"]);
                self.element(&HtmlToken::start("colgroup"), false, true);
                self.mode = InColumnGroup;
                return true;
            }
            _ if start && matches!(tag, "td" | "th" | "tr") => {
                self.clear_to(&["table", "template", "html"]);
                self.element(&HtmlToken::start("tbody"), false, true);
                self.mode = InTableBody;
                return true;
            }
            _ if tag == "table" => {
                if self.close_in_scope(&["table"], Scope::Table) {
                    self.reset_mode();
                    return start;
                }
            }
            _ if end
                && matches!(
                    tag,
                    "body"
                        | "caption"
                        | "col"
                        | "colgroup"
                        | "html"
                        | "tbody"
                        | "td"
                        | "tfoot"
                        | "th"
                        | "thead"
                        | "tr"
                ) => {}
            _ if start && matches!(tag, "style" | "script") => self.raw_element(token, false),
            _ if tag == "template" => self.in_body(token, false),
            HtmlToken::Start { attrs, .. }
                if tag == "input"
                    && attrs
                        .get("type")
                        .is_some_and(|value| value.eq_ignore_ascii_case("hidden")) =>
            {
                self.element(token, false, false);
            }
            _ if start && tag == "form" => {
                if self.form.is_none() || self.in_template() {
                    let id = self.element(token, false, false);
                    if !self.in_template() {
                        self.form = Some(id);
                    }
                }
            }
            _ => self.in_body(token, true),
        }
        false
    }
    fn in_body(&mut self, token: &HtmlToken, foster: bool) {
        use InsertionMode::*;
        match token {
            HtmlToken::Characters(text) => {
                if !text.bytes().all(is_space) {
                    self.frameset_ok = false;
                }
                if !text.is_empty() {
                    self.reconstruct_formatting(foster);
                }
                if self.work > 0 {
                    self.text(text, foster);
                }
            }
            HtmlToken::Comment(text) => self.comment(text, None),
            HtmlToken::Doctype(_) | HtmlToken::Eof | HtmlToken::ProcessingInstruction { .. } => {}
            HtmlToken::Start {
                tag,
                attrs,
                self_closing,
            } => {
                let tag = tag.as_str();
                if tag == "template" {
                    self.start_template(token);
                    return;
                }
                if matches!(tag, "input" | "select")
                    && self
                        .fragment_context
                        .is_some_and(|id| self.html_tag(id) == Some("select"))
                {
                    return;
                }
                if matches!(tag, "input" | "select")
                    && let Some(index) = self.scope(&["select"], Scope::Regular)
                {
                    self.truncate_open(index);
                    if tag == "select" {
                        return;
                    }
                }
                if tag == "frameset" {
                    if self.stack.len() < 2
                        || self.html_tag(self.stack[1]) != Some("body")
                        || !self.frameset_ok
                    {
                        return;
                    }
                    let body = self.stack[1];
                    if let Some(parent) = self.doc.nodes[body].parent {
                        self.doc.remove_child(parent, body);
                    }
                    self.truncate_open(1);
                    self.element(token, false, true);
                    self.mode = InFrameset;
                    return;
                }
                if tag == "html" {
                    if !self.in_template() {
                        self.merge_attrs(self.html, token);
                    }
                    return;
                }
                if tag == "body" {
                    if !self.in_template() {
                        self.frameset_ok = false;
                        self.merge_attrs(self.body, token);
                    }
                    return;
                }
                if tag == "head"
                    || matches!(
                        tag,
                        "caption"
                            | "col"
                            | "colgroup"
                            | "tbody"
                            | "td"
                            | "tfoot"
                            | "th"
                            | "thead"
                            | "tr"
                            | "frame"
                    )
                {
                    return;
                }
                if tag == "form" && self.form.is_some() && !self.in_template() {
                    return;
                }
                if matches!(
                    tag,
                    "pre"
                        | "listing"
                        | "li"
                        | "dd"
                        | "dt"
                        | "button"
                        | "applet"
                        | "marquee"
                        | "object"
                        | "table"
                        | "area"
                        | "br"
                        | "embed"
                        | "img"
                        | "image"
                        | "keygen"
                        | "wbr"
                        | "hr"
                        | "textarea"
                        | "xmp"
                        | "iframe"
                        | "select"
                ) || tag == "input"
                    && !attrs
                        .get("type")
                        .is_some_and(|value| value.eq_ignore_ascii_case("hidden"))
                {
                    self.frameset_ok = false;
                }
                if is_formatting(tag) {
                    if tag == "a"
                        && let Some(index) = self.formatting_index("a")
                    {
                        let previous = self.formatting[index];
                        self.adoption_agency("a", foster);
                        if !self.spend(self.formatting.len() + self.stack.len()) {
                            return;
                        }
                        self.formatting.retain(|entry| *entry != previous);
                        self.stack.retain(|id| Some(*id) != previous);
                    }
                    self.reconstruct_formatting(foster);
                    if tag == "nobr"
                        && self
                            .stack
                            .iter()
                            .rev()
                            .find(|node| self.html_tag(**node) == Some("nobr"))
                            .is_some_and(|node| self.node_in_scope(*node))
                    {
                        self.adoption_agency("nobr", foster);
                        self.reconstruct_formatting(foster);
                    }
                    if self.work == 0 {
                        return;
                    }
                    let id = self.element(token, foster, true);
                    self.push_formatting(id);
                    return;
                }
                if matches!(tag, "li" | "dd" | "dt") {
                    self.close_list_item(if tag == "li" { &["li"] } else { &["dd", "dt"] });
                }
                if closes_p(tag) && !(tag == "table" && self.doc.mode == DocumentMode::Quirks) {
                    self.close_in_scope(&["p"], Scope::Button);
                }
                if tag == "button" {
                    self.close_in_scope(&["button"], Scope::Regular);
                }
                if matches!(tag, "option" | "optgroup") {
                    if self.scope(&["select"], Scope::Regular).is_some() {
                        self.implied_end_tags((tag == "option").then_some("optgroup"));
                    } else if self.current_tag() == "option" {
                        self.pop_open();
                    }
                }
                if tag == "hr" && self.scope(&["select"], Scope::Regular).is_some() {
                    self.implied_end_tags(None);
                }
                if matches!(tag, "rb" | "rtc" | "rp" | "rt")
                    && self.scope(&["ruby"], Scope::Regular).is_some()
                {
                    self.implied_end_tags(matches!(tag, "rp" | "rt").then_some("rtc"));
                }
                if matches!(tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
                    && matches!(self.current_tag(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
                {
                    self.pop_open();
                }
                if tag == "image" {
                    self.reconstruct_formatting(foster);
                    if self.work == 0 {
                        return;
                    }
                    self.element(&HtmlToken::start("img"), foster, false);
                    return;
                }
                if matches!(
                    tag,
                    "script"
                        | "style"
                        | "title"
                        | "textarea"
                        | "xmp"
                        | "iframe"
                        | "noembed"
                        | "noframes"
                ) || tag == "noscript" && self.scripting
                {
                    if tag == "xmp" {
                        self.close_in_scope(&["p"], Scope::Button);
                        self.reconstruct_formatting(foster);
                    }
                    if self.work == 0 {
                        return;
                    }
                    self.raw_element(token, foster);
                    return;
                }
                if tag == "plaintext" {
                    self.element(token, foster, true);
                    self.raw = Some((String::new(), false));
                    return;
                }
                if reconstruct_before_start(tag) {
                    self.reconstruct_formatting(foster);
                    if self.work == 0 {
                        return;
                    }
                }
                if matches!(tag, "svg" | "math") {
                    let namespace = if tag == "svg" {
                        Namespace::Svg
                    } else {
                        Namespace::MathMl
                    };
                    self.element_ns(token, namespace, foster, !self_closing);
                    return;
                }
                let id = self.element(token, foster, !is_void(tag));
                if tag == "form" && !self.in_template() {
                    self.form = Some(id);
                }
                if matches!(tag, "pre" | "listing") {
                    self.skip_lf = true;
                }
                if tag == "table" {
                    self.mode = InTable;
                }
            }
            HtmlToken::End(tag) => {
                let tag = tag.as_str();
                if is_formatting(tag) {
                    self.adoption_agency(tag, foster);
                    return;
                }
                if matches!(tag, "applet" | "marquee" | "object") {
                    self.close_in_scope(&[tag], Scope::Regular);
                    return;
                }
                if matches!(tag, "body" | "html") {
                    if self.scope(&["body"], Scope::Regular).is_some() {
                        self.mode = if tag == "html" && self.fragment_context.is_none() {
                            AfterAfterBody
                        } else {
                            AfterBody
                        };
                    }
                    return;
                }
                if tag == "p" {
                    if self.scope(&["p"], Scope::Button).is_none() {
                        self.element(&HtmlToken::start("p"), foster, true);
                    }
                    self.close_in_scope(&["p"], Scope::Button);
                    return;
                }
                if tag == "br" {
                    self.reconstruct_formatting(foster);
                    if self.work == 0 {
                        return;
                    }
                    self.element(&HtmlToken::start("br"), foster, false);
                    return;
                }
                if tag == "form" {
                    if self.in_template() {
                        self.close_in_scope(&["form"], Scope::Regular);
                    } else if let Some(form) = self.form.take()
                        && let Some(index) = self.stack.iter().position(|id| *id == form)
                    {
                        self.stack.remove(index);
                    }
                    return;
                }
                if tag == "template" {
                    self.close_template();
                    return;
                }
                if tag == "li" {
                    self.close_in_scope(&["li"], Scope::ListItem);
                    return;
                }
                if matches!(tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                    self.close_in_scope(&["h1", "h2", "h3", "h4", "h5", "h6"], Scope::Regular);
                    return;
                }
                if matches!(
                    tag,
                    "address"
                        | "article"
                        | "aside"
                        | "blockquote"
                        | "button"
                        | "center"
                        | "details"
                        | "dialog"
                        | "dir"
                        | "div"
                        | "dl"
                        | "fieldset"
                        | "figcaption"
                        | "figure"
                        | "footer"
                        | "header"
                        | "hgroup"
                        | "listing"
                        | "main"
                        | "menu"
                        | "nav"
                        | "ol"
                        | "pre"
                        | "search"
                        | "section"
                        | "select"
                        | "summary"
                        | "ul"
                        | "dd"
                        | "dt"
                ) {
                    self.close_in_scope(&[tag], Scope::Regular);
                    return;
                }
                self.generic_end(tag);
            }
        }
    }
}
fn foreign_breakout(token: &HtmlToken) -> bool {
    match token {
        HtmlToken::End(tag) => matches!(tag.as_str(), "br" | "p"),
        HtmlToken::Start { tag, attrs, .. } => {
            matches!(
                tag.as_str(),
                "b" | "big"
                    | "blockquote"
                    | "body"
                    | "br"
                    | "center"
                    | "code"
                    | "dd"
                    | "div"
                    | "dl"
                    | "dt"
                    | "em"
                    | "embed"
                    | "h1"
                    | "h2"
                    | "h3"
                    | "h4"
                    | "h5"
                    | "h6"
                    | "head"
                    | "hr"
                    | "i"
                    | "img"
                    | "li"
                    | "listing"
                    | "menu"
                    | "meta"
                    | "nobr"
                    | "ol"
                    | "p"
                    | "pre"
                    | "ruby"
                    | "s"
                    | "small"
                    | "span"
                    | "strong"
                    | "strike"
                    | "sub"
                    | "sup"
                    | "table"
                    | "tt"
                    | "u"
                    | "ul"
                    | "var"
            ) || tag == "font"
                && ["color", "face", "size"]
                    .iter()
                    .any(|name| attrs.contains_key(*name))
        }
        _ => false,
    }
}
fn adjust_svg_tag(tag: &str) -> &str {
    match tag {
        "altglyph" => "altGlyph",
        "altglyphdef" => "altGlyphDef",
        "altglyphitem" => "altGlyphItem",
        "animatecolor" => "animateColor",
        "animatemotion" => "animateMotion",
        "animatetransform" => "animateTransform",
        "clippath" => "clipPath",
        "feblend" => "feBlend",
        "fecolormatrix" => "feColorMatrix",
        "fecomponenttransfer" => "feComponentTransfer",
        "fecomposite" => "feComposite",
        "feconvolvematrix" => "feConvolveMatrix",
        "fediffuselighting" => "feDiffuseLighting",
        "fedisplacementmap" => "feDisplacementMap",
        "fedistantlight" => "feDistantLight",
        "fedropshadow" => "feDropShadow",
        "feflood" => "feFlood",
        "fefunca" => "feFuncA",
        "fefuncb" => "feFuncB",
        "fefuncg" => "feFuncG",
        "fefuncr" => "feFuncR",
        "fegaussianblur" => "feGaussianBlur",
        "feimage" => "feImage",
        "femerge" => "feMerge",
        "femergenode" => "feMergeNode",
        "femorphology" => "feMorphology",
        "feoffset" => "feOffset",
        "fepointlight" => "fePointLight",
        "fespecularlighting" => "feSpecularLighting",
        "fespotlight" => "feSpotLight",
        "fetile" => "feTile",
        "feturbulence" => "feTurbulence",
        "foreignobject" => "foreignObject",
        "glyphref" => "glyphRef",
        "lineargradient" => "linearGradient",
        "radialgradient" => "radialGradient",
        "textpath" => "textPath",
        _ => tag,
    }
}
fn adjust_svg_attribute(name: &str) -> &str {
    match name {
        "attributename" => "attributeName",
        "attributetype" => "attributeType",
        "basefrequency" => "baseFrequency",
        "baseprofile" => "baseProfile",
        "calcmode" => "calcMode",
        "clippathunits" => "clipPathUnits",
        "diffuseconstant" => "diffuseConstant",
        "edgemode" => "edgeMode",
        "filterunits" => "filterUnits",
        "glyphref" => "glyphRef",
        "gradienttransform" => "gradientTransform",
        "gradientunits" => "gradientUnits",
        "kernelmatrix" => "kernelMatrix",
        "kernelunitlength" => "kernelUnitLength",
        "keypoints" => "keyPoints",
        "keysplines" => "keySplines",
        "keytimes" => "keyTimes",
        "lengthadjust" => "lengthAdjust",
        "limitingconeangle" => "limitingConeAngle",
        "markerheight" => "markerHeight",
        "markerunits" => "markerUnits",
        "markerwidth" => "markerWidth",
        "maskcontentunits" => "maskContentUnits",
        "maskunits" => "maskUnits",
        "numoctaves" => "numOctaves",
        "pathlength" => "pathLength",
        "patterncontentunits" => "patternContentUnits",
        "patterntransform" => "patternTransform",
        "patternunits" => "patternUnits",
        "pointsatx" => "pointsAtX",
        "pointsaty" => "pointsAtY",
        "pointsatz" => "pointsAtZ",
        "preservealpha" => "preserveAlpha",
        "preserveaspectratio" => "preserveAspectRatio",
        "primitiveunits" => "primitiveUnits",
        "refx" => "refX",
        "refy" => "refY",
        "repeatcount" => "repeatCount",
        "repeatdur" => "repeatDur",
        "requiredextensions" => "requiredExtensions",
        "requiredfeatures" => "requiredFeatures",
        "specularconstant" => "specularConstant",
        "specularexponent" => "specularExponent",
        "spreadmethod" => "spreadMethod",
        "startoffset" => "startOffset",
        "stddeviation" => "stdDeviation",
        "stitchtiles" => "stitchTiles",
        "surfacescale" => "surfaceScale",
        "systemlanguage" => "systemLanguage",
        "tablevalues" => "tableValues",
        "targetx" => "targetX",
        "targety" => "targetY",
        "textlength" => "textLength",
        "viewbox" => "viewBox",
        "viewtarget" => "viewTarget",
        "xchannelselector" => "xChannelSelector",
        "ychannelselector" => "yChannelSelector",
        "zoomandpan" => "zoomAndPan",
        _ => name,
    }
}
fn is_formatting(tag: &str) -> bool {
    matches!(
        tag,
        "a" | "b"
            | "big"
            | "code"
            | "em"
            | "font"
            | "i"
            | "nobr"
            | "s"
            | "small"
            | "strike"
            | "strong"
            | "tt"
            | "u"
    )
}
fn formatting_marker(tag: &str) -> bool {
    matches!(
        tag,
        "applet" | "object" | "marquee" | "template" | "td" | "th" | "caption"
    )
}
fn reconstruct_before_start(tag: &str) -> bool {
    !matches!(
        tag,
        "base"
            | "basefont"
            | "bgsound"
            | "link"
            | "meta"
            | "template"
            | "address"
            | "article"
            | "aside"
            | "blockquote"
            | "center"
            | "details"
            | "dialog"
            | "dir"
            | "div"
            | "dl"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "header"
            | "hgroup"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "p"
            | "search"
            | "section"
            | "summary"
            | "ul"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "pre"
            | "listing"
            | "form"
            | "li"
            | "dd"
            | "dt"
            | "table"
            | "param"
            | "source"
            | "track"
            | "hr"
            | "rb"
            | "rtc"
            | "rp"
            | "rt"
            | "frameset"
            | "frame"
    )
}
fn special_html(tag: &str) -> bool {
    closes_p(tag)
        || matches!(
            tag,
            "html"
                | "head"
                | "body"
                | "applet"
                | "button"
                | "caption"
                | "col"
                | "colgroup"
                | "dd"
                | "dt"
                | "li"
                | "marquee"
                | "object"
                | "select"
                | "tbody"
                | "td"
                | "tfoot"
                | "th"
                | "thead"
                | "tr"
                | "template"
                | "area"
                | "base"
                | "basefont"
                | "bgsound"
                | "br"
                | "center"
                | "details"
                | "dir"
                | "embed"
                | "figcaption"
                | "figure"
                | "frame"
                | "frameset"
                | "iframe"
                | "img"
                | "input"
                | "keygen"
                | "link"
                | "listing"
                | "menu"
                | "meta"
                | "noembed"
                | "noframes"
                | "noscript"
                | "param"
                | "plaintext"
                | "script"
                | "search"
                | "source"
                | "style"
                | "summary"
                | "textarea"
                | "title"
                | "track"
                | "wbr"
                | "xmp"
        )
}

/// WHATWG initial insertion-mode document compatibility classification.
fn doctype_mode(doctype: &Doctype) -> DocumentMode {
    let public = doctype.public_id.as_deref().unwrap_or("");
    let system = doctype.system_id.as_deref().unwrap_or("");
    let starts = |prefix: &str| {
        public
            .get(..prefix.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(prefix))
    };
    let html4 = starts("-//W3C//DTD HTML 4.01 Frameset//")
        || starts("-//W3C//DTD HTML 4.01 Transitional//");
    const QUIRKS_PREFIXES: &[&str] = &[
        "+//Silmaril//dtd html Pro v0r11 19970101//",
        "-//AS//DTD HTML 3.0 asWedit + extensions//",
        "-//AdvaSoft Ltd//DTD HTML 3.0 asWedit + extensions//",
        "-//IETF//DTD HTML 2.0 Level 1//",
        "-//IETF//DTD HTML 2.0 Level 2//",
        "-//IETF//DTD HTML 2.0 Strict Level 1//",
        "-//IETF//DTD HTML 2.0 Strict Level 2//",
        "-//IETF//DTD HTML 2.0 Strict//",
        "-//IETF//DTD HTML 2.0//",
        "-//IETF//DTD HTML 2.1E//",
        "-//IETF//DTD HTML 3.0//",
        "-//IETF//DTD HTML 3.2 Final//",
        "-//IETF//DTD HTML 3.2//",
        "-//IETF//DTD HTML 3//",
        "-//IETF//DTD HTML Level 0//",
        "-//IETF//DTD HTML Level 1//",
        "-//IETF//DTD HTML Level 2//",
        "-//IETF//DTD HTML Level 3//",
        "-//IETF//DTD HTML Strict Level 0//",
        "-//IETF//DTD HTML Strict Level 1//",
        "-//IETF//DTD HTML Strict Level 2//",
        "-//IETF//DTD HTML Strict Level 3//",
        "-//IETF//DTD HTML Strict//",
        "-//IETF//DTD HTML//",
        "-//Metrius//DTD Metrius Presentational//",
        "-//Microsoft//DTD Internet Explorer 2.0 HTML Strict//",
        "-//Microsoft//DTD Internet Explorer 2.0 HTML//",
        "-//Microsoft//DTD Internet Explorer 2.0 Tables//",
        "-//Microsoft//DTD Internet Explorer 3.0 HTML Strict//",
        "-//Microsoft//DTD Internet Explorer 3.0 HTML//",
        "-//Microsoft//DTD Internet Explorer 3.0 Tables//",
        "-//Netscape Comm. Corp.//DTD HTML//",
        "-//Netscape Comm. Corp.//DTD Strict HTML//",
        "-//O'Reilly and Associates//DTD HTML 2.0//",
        "-//O'Reilly and Associates//DTD HTML Extended 1.0//",
        "-//O'Reilly and Associates//DTD HTML Extended Relaxed 1.0//",
        "-//SQ//DTD HTML 2.0 HoTMetaL + extensions//",
        "-//SoftQuad Software//DTD HoTMetaL PRO 6.0::19990601::extensions to HTML 4.0//",
        "-//SoftQuad//DTD HoTMetaL PRO 4.0::19971010::extensions to HTML 4.0//",
        "-//Spyglass//DTD HTML 2.0 Extended//",
        "-//Sun Microsystems Corp.//DTD HotJava HTML//",
        "-//Sun Microsystems Corp.//DTD HotJava Strict HTML//",
        "-//W3C//DTD HTML 3 1995-03-24//",
        "-//W3C//DTD HTML 3.2 Draft//",
        "-//W3C//DTD HTML 3.2 Final//",
        "-//W3C//DTD HTML 3.2//",
        "-//W3C//DTD HTML 3.2S Draft//",
        "-//W3C//DTD HTML 4.0 Frameset//",
        "-//W3C//DTD HTML 4.0 Transitional//",
        "-//W3C//DTD HTML Experimental 19960712//",
        "-//W3C//DTD HTML Experimental 970421//",
        "-//W3C//DTD W3 HTML//",
        "-//W3O//DTD W3 HTML 3.0//",
        "-//WebTechs//DTD Mozilla HTML 2.0//",
        "-//WebTechs//DTD Mozilla HTML//",
    ];
    if doctype.force_quirks
        || doctype.name != "html"
        || [
            "-//W3O//DTD W3 HTML Strict 3.0//EN//",
            "-/W3C/DTD HTML 4.0 Transitional/EN",
            "HTML",
        ]
        .iter()
        .any(|id| public.eq_ignore_ascii_case(id))
        || system.eq_ignore_ascii_case("http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd")
        || QUIRKS_PREFIXES.iter().any(|prefix| starts(prefix))
        || html4 && system.is_empty()
    {
        DocumentMode::Quirks
    } else if html4
        || starts("-//W3C//DTD XHTML 1.0 Frameset//")
        || starts("-//W3C//DTD XHTML 1.0 Transitional//")
    {
        DocumentMode::LimitedQuirks
    } else {
        DocumentMode::NoQuirks
    }
}

pub fn parse(source: &str) -> Document {
    parse_with_scripting(source, false)
}
pub fn parse_with_scripting(source: &str, scripting: bool) -> Document {
    parse_with_builder(source, TreeBuilder::new(scripting))
}
fn parse_with_builder(source: &str, mut builder: TreeBuilder) -> Document {
    let source = &source[..floor_boundary(source, source.len().min(MAX_TEXT))];
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut tokenizer = HtmlTokenizer::new(&normalized);
    loop {
        tokenizer.raw = builder.raw.take();
        let skip_lf = std::mem::take(&mut builder.skip_lf);
        tokenizer.foreign = builder.foreign();
        tokenizer.foreign_characters = builder.foreign_characters();
        let mut token = tokenizer.next();
        if skip_lf
            && let HtmlToken::Characters(text) = &mut token
            && text.starts_with('\n')
        {
            text.remove(0);
        }
        let eof = matches!(token, HtmlToken::Eof);
        builder.process(token);
        if eof || builder.work == 0 || builder.doc.nodes.len() >= MAX_NODES {
            break;
        }
    }
    builder.truncate_open(0);
    builder.doc.initialize_base_tracking();
    builder.doc
}

pub fn decode_entities(s: &str) -> String {
    decode_entities_context(s, false)
}
fn decode_entities_context(s: &str, in_attribute: bool) -> String {
    decode_entities_in_state(s, in_attribute, true)
}
fn decode_entities_in_state(s: &str, in_attribute: bool, replace_null: bool) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'&' {
            let c = s[i..].chars().next().unwrap_or('\u{fffd}');
            out.push(if c == '\0' && replace_null {
                '\u{fffd}'
            } else {
                c
            });
            i += c.len_utf8();
            continue;
        }
        let start = i + 1;
        if bytes.get(start) == Some(&b'#') {
            let mut end = start + 1;
            let radix = if matches!(bytes.get(end), Some(b'x' | b'X')) {
                end += 1;
                16
            } else {
                10
            };
            let number_start = end;
            while bytes.get(end).is_some_and(|b| {
                if radix == 16 {
                    b.is_ascii_hexdigit()
                } else {
                    b.is_ascii_digit()
                }
            }) {
                end += 1;
            }
            if end > number_start {
                let value = u32::from_str_radix(&s[number_start..end], radix).unwrap_or(0xfffd);
                out.push(normalize_numeric_entity(value));
                i = end + usize::from(bytes.get(end) == Some(&b';'));
                continue;
            }
        } else {
            let mut end = start;
            while end - start < 32 && bytes.get(end).is_some_and(u8::is_ascii_alphanumeric) {
                end += 1;
            }
            if bytes.get(end) == Some(&b';') {
                end += 1;
            }
            let mut found = None;
            for candidate_end in (start + 1..=end).rev() {
                let name = &s[start..candidate_end];
                if let Some(value) = named_entity(name) {
                    if in_attribute
                        && !name.ends_with(';')
                        && bytes
                            .get(candidate_end)
                            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'=')
                    {
                        break;
                    }
                    found = Some((candidate_end, value));
                    break;
                }
            }
            if let Some((end, value)) = found {
                out.push_str(value);
                i = end;
                continue;
            }
        }
        out.push('&');
        i += 1;
    }
    out
}
fn normalize_numeric_entity(value: u32) -> char {
    let value = match value {
        0 => 0xfffd,
        0x80 => 0x20ac,
        0x82 => 0x201a,
        0x83 => 0x192,
        0x84 => 0x201e,
        0x85 => 0x2026,
        0x86 => 0x2020,
        0x87 => 0x2021,
        0x88 => 0x2c6,
        0x89 => 0x2030,
        0x8a => 0x160,
        0x8b => 0x2039,
        0x8c => 0x152,
        0x8e => 0x17d,
        0x91 => 0x2018,
        0x92 => 0x2019,
        0x93 => 0x201c,
        0x94 => 0x201d,
        0x95 => 0x2022,
        0x96 => 0x2013,
        0x97 => 0x2014,
        0x98 => 0x2dc,
        0x99 => 0x2122,
        0x9a => 0x161,
        0x9b => 0x203a,
        0x9c => 0x153,
        0x9e => 0x17e,
        0x9f => 0x178,
        _ => value,
    };
    char::from_u32(value).unwrap_or('\u{fffd}')
}
// Standard HTML named character reference data, including legacy semicolon omissions.
fn named_entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "AElig" => "\u{c6}",
        "AElig;" => "\u{c6}",
        "AMP" => "\u{26}",
        "AMP;" => "\u{26}",
        "Aacute" => "\u{c1}",
        "Aacute;" => "\u{c1}",
        "Abreve;" => "\u{102}",
        "Acirc" => "\u{c2}",
        "Acirc;" => "\u{c2}",
        "Acy;" => "\u{410}",
        "Afr;" => "\u{1d504}",
        "Agrave" => "\u{c0}",
        "Agrave;" => "\u{c0}",
        "Alpha;" => "\u{391}",
        "Amacr;" => "\u{100}",
        "And;" => "\u{2a53}",
        "Aogon;" => "\u{104}",
        "Aopf;" => "\u{1d538}",
        "ApplyFunction;" => "\u{2061}",
        "Aring" => "\u{c5}",
        "Aring;" => "\u{c5}",
        "Ascr;" => "\u{1d49c}",
        "Assign;" => "\u{2254}",
        "Atilde" => "\u{c3}",
        "Atilde;" => "\u{c3}",
        "Auml" => "\u{c4}",
        "Auml;" => "\u{c4}",
        "Backslash;" => "\u{2216}",
        "Barv;" => "\u{2ae7}",
        "Barwed;" => "\u{2306}",
        "Bcy;" => "\u{411}",
        "Because;" => "\u{2235}",
        "Bernoullis;" => "\u{212c}",
        "Beta;" => "\u{392}",
        "Bfr;" => "\u{1d505}",
        "Bopf;" => "\u{1d539}",
        "Breve;" => "\u{2d8}",
        "Bscr;" => "\u{212c}",
        "Bumpeq;" => "\u{224e}",
        "CHcy;" => "\u{427}",
        "COPY" => "\u{a9}",
        "COPY;" => "\u{a9}",
        "Cacute;" => "\u{106}",
        "Cap;" => "\u{22d2}",
        "CapitalDifferentialD;" => "\u{2145}",
        "Cayleys;" => "\u{212d}",
        "Ccaron;" => "\u{10c}",
        "Ccedil" => "\u{c7}",
        "Ccedil;" => "\u{c7}",
        "Ccirc;" => "\u{108}",
        "Cconint;" => "\u{2230}",
        "Cdot;" => "\u{10a}",
        "Cedilla;" => "\u{b8}",
        "CenterDot;" => "\u{b7}",
        "Cfr;" => "\u{212d}",
        "Chi;" => "\u{3a7}",
        "CircleDot;" => "\u{2299}",
        "CircleMinus;" => "\u{2296}",
        "CirclePlus;" => "\u{2295}",
        "CircleTimes;" => "\u{2297}",
        "ClockwiseContourIntegral;" => "\u{2232}",
        "CloseCurlyDoubleQuote;" => "\u{201d}",
        "CloseCurlyQuote;" => "\u{2019}",
        "Colon;" => "\u{2237}",
        "Colone;" => "\u{2a74}",
        "Congruent;" => "\u{2261}",
        "Conint;" => "\u{222f}",
        "ContourIntegral;" => "\u{222e}",
        "Copf;" => "\u{2102}",
        "Coproduct;" => "\u{2210}",
        "CounterClockwiseContourIntegral;" => "\u{2233}",
        "Cross;" => "\u{2a2f}",
        "Cscr;" => "\u{1d49e}",
        "Cup;" => "\u{22d3}",
        "CupCap;" => "\u{224d}",
        "DD;" => "\u{2145}",
        "DDotrahd;" => "\u{2911}",
        "DJcy;" => "\u{402}",
        "DScy;" => "\u{405}",
        "DZcy;" => "\u{40f}",
        "Dagger;" => "\u{2021}",
        "Darr;" => "\u{21a1}",
        "Dashv;" => "\u{2ae4}",
        "Dcaron;" => "\u{10e}",
        "Dcy;" => "\u{414}",
        "Del;" => "\u{2207}",
        "Delta;" => "\u{394}",
        "Dfr;" => "\u{1d507}",
        "DiacriticalAcute;" => "\u{b4}",
        "DiacriticalDot;" => "\u{2d9}",
        "DiacriticalDoubleAcute;" => "\u{2dd}",
        "DiacriticalGrave;" => "\u{60}",
        "DiacriticalTilde;" => "\u{2dc}",
        "Diamond;" => "\u{22c4}",
        "DifferentialD;" => "\u{2146}",
        "Dopf;" => "\u{1d53b}",
        "Dot;" => "\u{a8}",
        "DotDot;" => "\u{20dc}",
        "DotEqual;" => "\u{2250}",
        "DoubleContourIntegral;" => "\u{222f}",
        "DoubleDot;" => "\u{a8}",
        "DoubleDownArrow;" => "\u{21d3}",
        "DoubleLeftArrow;" => "\u{21d0}",
        "DoubleLeftRightArrow;" => "\u{21d4}",
        "DoubleLeftTee;" => "\u{2ae4}",
        "DoubleLongLeftArrow;" => "\u{27f8}",
        "DoubleLongLeftRightArrow;" => "\u{27fa}",
        "DoubleLongRightArrow;" => "\u{27f9}",
        "DoubleRightArrow;" => "\u{21d2}",
        "DoubleRightTee;" => "\u{22a8}",
        "DoubleUpArrow;" => "\u{21d1}",
        "DoubleUpDownArrow;" => "\u{21d5}",
        "DoubleVerticalBar;" => "\u{2225}",
        "DownArrow;" => "\u{2193}",
        "DownArrowBar;" => "\u{2913}",
        "DownArrowUpArrow;" => "\u{21f5}",
        "DownBreve;" => "\u{311}",
        "DownLeftRightVector;" => "\u{2950}",
        "DownLeftTeeVector;" => "\u{295e}",
        "DownLeftVector;" => "\u{21bd}",
        "DownLeftVectorBar;" => "\u{2956}",
        "DownRightTeeVector;" => "\u{295f}",
        "DownRightVector;" => "\u{21c1}",
        "DownRightVectorBar;" => "\u{2957}",
        "DownTee;" => "\u{22a4}",
        "DownTeeArrow;" => "\u{21a7}",
        "Downarrow;" => "\u{21d3}",
        "Dscr;" => "\u{1d49f}",
        "Dstrok;" => "\u{110}",
        "ENG;" => "\u{14a}",
        "ETH" => "\u{d0}",
        "ETH;" => "\u{d0}",
        "Eacute" => "\u{c9}",
        "Eacute;" => "\u{c9}",
        "Ecaron;" => "\u{11a}",
        "Ecirc" => "\u{ca}",
        "Ecirc;" => "\u{ca}",
        "Ecy;" => "\u{42d}",
        "Edot;" => "\u{116}",
        "Efr;" => "\u{1d508}",
        "Egrave" => "\u{c8}",
        "Egrave;" => "\u{c8}",
        "Element;" => "\u{2208}",
        "Emacr;" => "\u{112}",
        "EmptySmallSquare;" => "\u{25fb}",
        "EmptyVerySmallSquare;" => "\u{25ab}",
        "Eogon;" => "\u{118}",
        "Eopf;" => "\u{1d53c}",
        "Epsilon;" => "\u{395}",
        "Equal;" => "\u{2a75}",
        "EqualTilde;" => "\u{2242}",
        "Equilibrium;" => "\u{21cc}",
        "Escr;" => "\u{2130}",
        "Esim;" => "\u{2a73}",
        "Eta;" => "\u{397}",
        "Euml" => "\u{cb}",
        "Euml;" => "\u{cb}",
        "Exists;" => "\u{2203}",
        "ExponentialE;" => "\u{2147}",
        "Fcy;" => "\u{424}",
        "Ffr;" => "\u{1d509}",
        "FilledSmallSquare;" => "\u{25fc}",
        "FilledVerySmallSquare;" => "\u{25aa}",
        "Fopf;" => "\u{1d53d}",
        "ForAll;" => "\u{2200}",
        "Fouriertrf;" => "\u{2131}",
        "Fscr;" => "\u{2131}",
        "GJcy;" => "\u{403}",
        "GT" => "\u{3e}",
        "GT;" => "\u{3e}",
        "Gamma;" => "\u{393}",
        "Gammad;" => "\u{3dc}",
        "Gbreve;" => "\u{11e}",
        "Gcedil;" => "\u{122}",
        "Gcirc;" => "\u{11c}",
        "Gcy;" => "\u{413}",
        "Gdot;" => "\u{120}",
        "Gfr;" => "\u{1d50a}",
        "Gg;" => "\u{22d9}",
        "Gopf;" => "\u{1d53e}",
        "GreaterEqual;" => "\u{2265}",
        "GreaterEqualLess;" => "\u{22db}",
        "GreaterFullEqual;" => "\u{2267}",
        "GreaterGreater;" => "\u{2aa2}",
        "GreaterLess;" => "\u{2277}",
        "GreaterSlantEqual;" => "\u{2a7e}",
        "GreaterTilde;" => "\u{2273}",
        "Gscr;" => "\u{1d4a2}",
        "Gt;" => "\u{226b}",
        "HARDcy;" => "\u{42a}",
        "Hacek;" => "\u{2c7}",
        "Hat;" => "\u{5e}",
        "Hcirc;" => "\u{124}",
        "Hfr;" => "\u{210c}",
        "HilbertSpace;" => "\u{210b}",
        "Hopf;" => "\u{210d}",
        "HorizontalLine;" => "\u{2500}",
        "Hscr;" => "\u{210b}",
        "Hstrok;" => "\u{126}",
        "HumpDownHump;" => "\u{224e}",
        "HumpEqual;" => "\u{224f}",
        "IEcy;" => "\u{415}",
        "IJlig;" => "\u{132}",
        "IOcy;" => "\u{401}",
        "Iacute" => "\u{cd}",
        "Iacute;" => "\u{cd}",
        "Icirc" => "\u{ce}",
        "Icirc;" => "\u{ce}",
        "Icy;" => "\u{418}",
        "Idot;" => "\u{130}",
        "Ifr;" => "\u{2111}",
        "Igrave" => "\u{cc}",
        "Igrave;" => "\u{cc}",
        "Im;" => "\u{2111}",
        "Imacr;" => "\u{12a}",
        "ImaginaryI;" => "\u{2148}",
        "Implies;" => "\u{21d2}",
        "Int;" => "\u{222c}",
        "Integral;" => "\u{222b}",
        "Intersection;" => "\u{22c2}",
        "InvisibleComma;" => "\u{2063}",
        "InvisibleTimes;" => "\u{2062}",
        "Iogon;" => "\u{12e}",
        "Iopf;" => "\u{1d540}",
        "Iota;" => "\u{399}",
        "Iscr;" => "\u{2110}",
        "Itilde;" => "\u{128}",
        "Iukcy;" => "\u{406}",
        "Iuml" => "\u{cf}",
        "Iuml;" => "\u{cf}",
        "Jcirc;" => "\u{134}",
        "Jcy;" => "\u{419}",
        "Jfr;" => "\u{1d50d}",
        "Jopf;" => "\u{1d541}",
        "Jscr;" => "\u{1d4a5}",
        "Jsercy;" => "\u{408}",
        "Jukcy;" => "\u{404}",
        "KHcy;" => "\u{425}",
        "KJcy;" => "\u{40c}",
        "Kappa;" => "\u{39a}",
        "Kcedil;" => "\u{136}",
        "Kcy;" => "\u{41a}",
        "Kfr;" => "\u{1d50e}",
        "Kopf;" => "\u{1d542}",
        "Kscr;" => "\u{1d4a6}",
        "LJcy;" => "\u{409}",
        "LT" => "\u{3c}",
        "LT;" => "\u{3c}",
        "Lacute;" => "\u{139}",
        "Lambda;" => "\u{39b}",
        "Lang;" => "\u{27ea}",
        "Laplacetrf;" => "\u{2112}",
        "Larr;" => "\u{219e}",
        "Lcaron;" => "\u{13d}",
        "Lcedil;" => "\u{13b}",
        "Lcy;" => "\u{41b}",
        "LeftAngleBracket;" => "\u{27e8}",
        "LeftArrow;" => "\u{2190}",
        "LeftArrowBar;" => "\u{21e4}",
        "LeftArrowRightArrow;" => "\u{21c6}",
        "LeftCeiling;" => "\u{2308}",
        "LeftDoubleBracket;" => "\u{27e6}",
        "LeftDownTeeVector;" => "\u{2961}",
        "LeftDownVector;" => "\u{21c3}",
        "LeftDownVectorBar;" => "\u{2959}",
        "LeftFloor;" => "\u{230a}",
        "LeftRightArrow;" => "\u{2194}",
        "LeftRightVector;" => "\u{294e}",
        "LeftTee;" => "\u{22a3}",
        "LeftTeeArrow;" => "\u{21a4}",
        "LeftTeeVector;" => "\u{295a}",
        "LeftTriangle;" => "\u{22b2}",
        "LeftTriangleBar;" => "\u{29cf}",
        "LeftTriangleEqual;" => "\u{22b4}",
        "LeftUpDownVector;" => "\u{2951}",
        "LeftUpTeeVector;" => "\u{2960}",
        "LeftUpVector;" => "\u{21bf}",
        "LeftUpVectorBar;" => "\u{2958}",
        "LeftVector;" => "\u{21bc}",
        "LeftVectorBar;" => "\u{2952}",
        "Leftarrow;" => "\u{21d0}",
        "Leftrightarrow;" => "\u{21d4}",
        "LessEqualGreater;" => "\u{22da}",
        "LessFullEqual;" => "\u{2266}",
        "LessGreater;" => "\u{2276}",
        "LessLess;" => "\u{2aa1}",
        "LessSlantEqual;" => "\u{2a7d}",
        "LessTilde;" => "\u{2272}",
        "Lfr;" => "\u{1d50f}",
        "Ll;" => "\u{22d8}",
        "Lleftarrow;" => "\u{21da}",
        "Lmidot;" => "\u{13f}",
        "LongLeftArrow;" => "\u{27f5}",
        "LongLeftRightArrow;" => "\u{27f7}",
        "LongRightArrow;" => "\u{27f6}",
        "Longleftarrow;" => "\u{27f8}",
        "Longleftrightarrow;" => "\u{27fa}",
        "Longrightarrow;" => "\u{27f9}",
        "Lopf;" => "\u{1d543}",
        "LowerLeftArrow;" => "\u{2199}",
        "LowerRightArrow;" => "\u{2198}",
        "Lscr;" => "\u{2112}",
        "Lsh;" => "\u{21b0}",
        "Lstrok;" => "\u{141}",
        "Lt;" => "\u{226a}",
        "Map;" => "\u{2905}",
        "Mcy;" => "\u{41c}",
        "MediumSpace;" => "\u{205f}",
        "Mellintrf;" => "\u{2133}",
        "Mfr;" => "\u{1d510}",
        "MinusPlus;" => "\u{2213}",
        "Mopf;" => "\u{1d544}",
        "Mscr;" => "\u{2133}",
        "Mu;" => "\u{39c}",
        "NJcy;" => "\u{40a}",
        "Nacute;" => "\u{143}",
        "Ncaron;" => "\u{147}",
        "Ncedil;" => "\u{145}",
        "Ncy;" => "\u{41d}",
        "NegativeMediumSpace;" => "\u{200b}",
        "NegativeThickSpace;" => "\u{200b}",
        "NegativeThinSpace;" => "\u{200b}",
        "NegativeVeryThinSpace;" => "\u{200b}",
        "NestedGreaterGreater;" => "\u{226b}",
        "NestedLessLess;" => "\u{226a}",
        "NewLine;" => "\u{a}",
        "Nfr;" => "\u{1d511}",
        "NoBreak;" => "\u{2060}",
        "NonBreakingSpace;" => "\u{a0}",
        "Nopf;" => "\u{2115}",
        "Not;" => "\u{2aec}",
        "NotCongruent;" => "\u{2262}",
        "NotCupCap;" => "\u{226d}",
        "NotDoubleVerticalBar;" => "\u{2226}",
        "NotElement;" => "\u{2209}",
        "NotEqual;" => "\u{2260}",
        "NotEqualTilde;" => "\u{2242}\u{338}",
        "NotExists;" => "\u{2204}",
        "NotGreater;" => "\u{226f}",
        "NotGreaterEqual;" => "\u{2271}",
        "NotGreaterFullEqual;" => "\u{2267}\u{338}",
        "NotGreaterGreater;" => "\u{226b}\u{338}",
        "NotGreaterLess;" => "\u{2279}",
        "NotGreaterSlantEqual;" => "\u{2a7e}\u{338}",
        "NotGreaterTilde;" => "\u{2275}",
        "NotHumpDownHump;" => "\u{224e}\u{338}",
        "NotHumpEqual;" => "\u{224f}\u{338}",
        "NotLeftTriangle;" => "\u{22ea}",
        "NotLeftTriangleBar;" => "\u{29cf}\u{338}",
        "NotLeftTriangleEqual;" => "\u{22ec}",
        "NotLess;" => "\u{226e}",
        "NotLessEqual;" => "\u{2270}",
        "NotLessGreater;" => "\u{2278}",
        "NotLessLess;" => "\u{226a}\u{338}",
        "NotLessSlantEqual;" => "\u{2a7d}\u{338}",
        "NotLessTilde;" => "\u{2274}",
        "NotNestedGreaterGreater;" => "\u{2aa2}\u{338}",
        "NotNestedLessLess;" => "\u{2aa1}\u{338}",
        "NotPrecedes;" => "\u{2280}",
        "NotPrecedesEqual;" => "\u{2aaf}\u{338}",
        "NotPrecedesSlantEqual;" => "\u{22e0}",
        "NotReverseElement;" => "\u{220c}",
        "NotRightTriangle;" => "\u{22eb}",
        "NotRightTriangleBar;" => "\u{29d0}\u{338}",
        "NotRightTriangleEqual;" => "\u{22ed}",
        "NotSquareSubset;" => "\u{228f}\u{338}",
        "NotSquareSubsetEqual;" => "\u{22e2}",
        "NotSquareSuperset;" => "\u{2290}\u{338}",
        "NotSquareSupersetEqual;" => "\u{22e3}",
        "NotSubset;" => "\u{2282}\u{20d2}",
        "NotSubsetEqual;" => "\u{2288}",
        "NotSucceeds;" => "\u{2281}",
        "NotSucceedsEqual;" => "\u{2ab0}\u{338}",
        "NotSucceedsSlantEqual;" => "\u{22e1}",
        "NotSucceedsTilde;" => "\u{227f}\u{338}",
        "NotSuperset;" => "\u{2283}\u{20d2}",
        "NotSupersetEqual;" => "\u{2289}",
        "NotTilde;" => "\u{2241}",
        "NotTildeEqual;" => "\u{2244}",
        "NotTildeFullEqual;" => "\u{2247}",
        "NotTildeTilde;" => "\u{2249}",
        "NotVerticalBar;" => "\u{2224}",
        "Nscr;" => "\u{1d4a9}",
        "Ntilde" => "\u{d1}",
        "Ntilde;" => "\u{d1}",
        "Nu;" => "\u{39d}",
        "OElig;" => "\u{152}",
        "Oacute" => "\u{d3}",
        "Oacute;" => "\u{d3}",
        "Ocirc" => "\u{d4}",
        "Ocirc;" => "\u{d4}",
        "Ocy;" => "\u{41e}",
        "Odblac;" => "\u{150}",
        "Ofr;" => "\u{1d512}",
        "Ograve" => "\u{d2}",
        "Ograve;" => "\u{d2}",
        "Omacr;" => "\u{14c}",
        "Omega;" => "\u{3a9}",
        "Omicron;" => "\u{39f}",
        "Oopf;" => "\u{1d546}",
        "OpenCurlyDoubleQuote;" => "\u{201c}",
        "OpenCurlyQuote;" => "\u{2018}",
        "Or;" => "\u{2a54}",
        "Oscr;" => "\u{1d4aa}",
        "Oslash" => "\u{d8}",
        "Oslash;" => "\u{d8}",
        "Otilde" => "\u{d5}",
        "Otilde;" => "\u{d5}",
        "Otimes;" => "\u{2a37}",
        "Ouml" => "\u{d6}",
        "Ouml;" => "\u{d6}",
        "OverBar;" => "\u{203e}",
        "OverBrace;" => "\u{23de}",
        "OverBracket;" => "\u{23b4}",
        "OverParenthesis;" => "\u{23dc}",
        "PartialD;" => "\u{2202}",
        "Pcy;" => "\u{41f}",
        "Pfr;" => "\u{1d513}",
        "Phi;" => "\u{3a6}",
        "Pi;" => "\u{3a0}",
        "PlusMinus;" => "\u{b1}",
        "Poincareplane;" => "\u{210c}",
        "Popf;" => "\u{2119}",
        "Pr;" => "\u{2abb}",
        "Precedes;" => "\u{227a}",
        "PrecedesEqual;" => "\u{2aaf}",
        "PrecedesSlantEqual;" => "\u{227c}",
        "PrecedesTilde;" => "\u{227e}",
        "Prime;" => "\u{2033}",
        "Product;" => "\u{220f}",
        "Proportion;" => "\u{2237}",
        "Proportional;" => "\u{221d}",
        "Pscr;" => "\u{1d4ab}",
        "Psi;" => "\u{3a8}",
        "QUOT" => "\u{22}",
        "QUOT;" => "\u{22}",
        "Qfr;" => "\u{1d514}",
        "Qopf;" => "\u{211a}",
        "Qscr;" => "\u{1d4ac}",
        "RBarr;" => "\u{2910}",
        "REG" => "\u{ae}",
        "REG;" => "\u{ae}",
        "Racute;" => "\u{154}",
        "Rang;" => "\u{27eb}",
        "Rarr;" => "\u{21a0}",
        "Rarrtl;" => "\u{2916}",
        "Rcaron;" => "\u{158}",
        "Rcedil;" => "\u{156}",
        "Rcy;" => "\u{420}",
        "Re;" => "\u{211c}",
        "ReverseElement;" => "\u{220b}",
        "ReverseEquilibrium;" => "\u{21cb}",
        "ReverseUpEquilibrium;" => "\u{296f}",
        "Rfr;" => "\u{211c}",
        "Rho;" => "\u{3a1}",
        "RightAngleBracket;" => "\u{27e9}",
        "RightArrow;" => "\u{2192}",
        "RightArrowBar;" => "\u{21e5}",
        "RightArrowLeftArrow;" => "\u{21c4}",
        "RightCeiling;" => "\u{2309}",
        "RightDoubleBracket;" => "\u{27e7}",
        "RightDownTeeVector;" => "\u{295d}",
        "RightDownVector;" => "\u{21c2}",
        "RightDownVectorBar;" => "\u{2955}",
        "RightFloor;" => "\u{230b}",
        "RightTee;" => "\u{22a2}",
        "RightTeeArrow;" => "\u{21a6}",
        "RightTeeVector;" => "\u{295b}",
        "RightTriangle;" => "\u{22b3}",
        "RightTriangleBar;" => "\u{29d0}",
        "RightTriangleEqual;" => "\u{22b5}",
        "RightUpDownVector;" => "\u{294f}",
        "RightUpTeeVector;" => "\u{295c}",
        "RightUpVector;" => "\u{21be}",
        "RightUpVectorBar;" => "\u{2954}",
        "RightVector;" => "\u{21c0}",
        "RightVectorBar;" => "\u{2953}",
        "Rightarrow;" => "\u{21d2}",
        "Ropf;" => "\u{211d}",
        "RoundImplies;" => "\u{2970}",
        "Rrightarrow;" => "\u{21db}",
        "Rscr;" => "\u{211b}",
        "Rsh;" => "\u{21b1}",
        "RuleDelayed;" => "\u{29f4}",
        "SHCHcy;" => "\u{429}",
        "SHcy;" => "\u{428}",
        "SOFTcy;" => "\u{42c}",
        "Sacute;" => "\u{15a}",
        "Sc;" => "\u{2abc}",
        "Scaron;" => "\u{160}",
        "Scedil;" => "\u{15e}",
        "Scirc;" => "\u{15c}",
        "Scy;" => "\u{421}",
        "Sfr;" => "\u{1d516}",
        "ShortDownArrow;" => "\u{2193}",
        "ShortLeftArrow;" => "\u{2190}",
        "ShortRightArrow;" => "\u{2192}",
        "ShortUpArrow;" => "\u{2191}",
        "Sigma;" => "\u{3a3}",
        "SmallCircle;" => "\u{2218}",
        "Sopf;" => "\u{1d54a}",
        "Sqrt;" => "\u{221a}",
        "Square;" => "\u{25a1}",
        "SquareIntersection;" => "\u{2293}",
        "SquareSubset;" => "\u{228f}",
        "SquareSubsetEqual;" => "\u{2291}",
        "SquareSuperset;" => "\u{2290}",
        "SquareSupersetEqual;" => "\u{2292}",
        "SquareUnion;" => "\u{2294}",
        "Sscr;" => "\u{1d4ae}",
        "Star;" => "\u{22c6}",
        "Sub;" => "\u{22d0}",
        "Subset;" => "\u{22d0}",
        "SubsetEqual;" => "\u{2286}",
        "Succeeds;" => "\u{227b}",
        "SucceedsEqual;" => "\u{2ab0}",
        "SucceedsSlantEqual;" => "\u{227d}",
        "SucceedsTilde;" => "\u{227f}",
        "SuchThat;" => "\u{220b}",
        "Sum;" => "\u{2211}",
        "Sup;" => "\u{22d1}",
        "Superset;" => "\u{2283}",
        "SupersetEqual;" => "\u{2287}",
        "Supset;" => "\u{22d1}",
        "THORN" => "\u{de}",
        "THORN;" => "\u{de}",
        "TRADE;" => "\u{2122}",
        "TSHcy;" => "\u{40b}",
        "TScy;" => "\u{426}",
        "Tab;" => "\u{9}",
        "Tau;" => "\u{3a4}",
        "Tcaron;" => "\u{164}",
        "Tcedil;" => "\u{162}",
        "Tcy;" => "\u{422}",
        "Tfr;" => "\u{1d517}",
        "Therefore;" => "\u{2234}",
        "Theta;" => "\u{398}",
        "ThickSpace;" => "\u{205f}\u{200a}",
        "ThinSpace;" => "\u{2009}",
        "Tilde;" => "\u{223c}",
        "TildeEqual;" => "\u{2243}",
        "TildeFullEqual;" => "\u{2245}",
        "TildeTilde;" => "\u{2248}",
        "Topf;" => "\u{1d54b}",
        "TripleDot;" => "\u{20db}",
        "Tscr;" => "\u{1d4af}",
        "Tstrok;" => "\u{166}",
        "Uacute" => "\u{da}",
        "Uacute;" => "\u{da}",
        "Uarr;" => "\u{219f}",
        "Uarrocir;" => "\u{2949}",
        "Ubrcy;" => "\u{40e}",
        "Ubreve;" => "\u{16c}",
        "Ucirc" => "\u{db}",
        "Ucirc;" => "\u{db}",
        "Ucy;" => "\u{423}",
        "Udblac;" => "\u{170}",
        "Ufr;" => "\u{1d518}",
        "Ugrave" => "\u{d9}",
        "Ugrave;" => "\u{d9}",
        "Umacr;" => "\u{16a}",
        "UnderBar;" => "\u{5f}",
        "UnderBrace;" => "\u{23df}",
        "UnderBracket;" => "\u{23b5}",
        "UnderParenthesis;" => "\u{23dd}",
        "Union;" => "\u{22c3}",
        "UnionPlus;" => "\u{228e}",
        "Uogon;" => "\u{172}",
        "Uopf;" => "\u{1d54c}",
        "UpArrow;" => "\u{2191}",
        "UpArrowBar;" => "\u{2912}",
        "UpArrowDownArrow;" => "\u{21c5}",
        "UpDownArrow;" => "\u{2195}",
        "UpEquilibrium;" => "\u{296e}",
        "UpTee;" => "\u{22a5}",
        "UpTeeArrow;" => "\u{21a5}",
        "Uparrow;" => "\u{21d1}",
        "Updownarrow;" => "\u{21d5}",
        "UpperLeftArrow;" => "\u{2196}",
        "UpperRightArrow;" => "\u{2197}",
        "Upsi;" => "\u{3d2}",
        "Upsilon;" => "\u{3a5}",
        "Uring;" => "\u{16e}",
        "Uscr;" => "\u{1d4b0}",
        "Utilde;" => "\u{168}",
        "Uuml" => "\u{dc}",
        "Uuml;" => "\u{dc}",
        "VDash;" => "\u{22ab}",
        "Vbar;" => "\u{2aeb}",
        "Vcy;" => "\u{412}",
        "Vdash;" => "\u{22a9}",
        "Vdashl;" => "\u{2ae6}",
        "Vee;" => "\u{22c1}",
        "Verbar;" => "\u{2016}",
        "Vert;" => "\u{2016}",
        "VerticalBar;" => "\u{2223}",
        "VerticalLine;" => "\u{7c}",
        "VerticalSeparator;" => "\u{2758}",
        "VerticalTilde;" => "\u{2240}",
        "VeryThinSpace;" => "\u{200a}",
        "Vfr;" => "\u{1d519}",
        "Vopf;" => "\u{1d54d}",
        "Vscr;" => "\u{1d4b1}",
        "Vvdash;" => "\u{22aa}",
        "Wcirc;" => "\u{174}",
        "Wedge;" => "\u{22c0}",
        "Wfr;" => "\u{1d51a}",
        "Wopf;" => "\u{1d54e}",
        "Wscr;" => "\u{1d4b2}",
        "Xfr;" => "\u{1d51b}",
        "Xi;" => "\u{39e}",
        "Xopf;" => "\u{1d54f}",
        "Xscr;" => "\u{1d4b3}",
        "YAcy;" => "\u{42f}",
        "YIcy;" => "\u{407}",
        "YUcy;" => "\u{42e}",
        "Yacute" => "\u{dd}",
        "Yacute;" => "\u{dd}",
        "Ycirc;" => "\u{176}",
        "Ycy;" => "\u{42b}",
        "Yfr;" => "\u{1d51c}",
        "Yopf;" => "\u{1d550}",
        "Yscr;" => "\u{1d4b4}",
        "Yuml;" => "\u{178}",
        "ZHcy;" => "\u{416}",
        "Zacute;" => "\u{179}",
        "Zcaron;" => "\u{17d}",
        "Zcy;" => "\u{417}",
        "Zdot;" => "\u{17b}",
        "ZeroWidthSpace;" => "\u{200b}",
        "Zeta;" => "\u{396}",
        "Zfr;" => "\u{2128}",
        "Zopf;" => "\u{2124}",
        "Zscr;" => "\u{1d4b5}",
        "aacute" => "\u{e1}",
        "aacute;" => "\u{e1}",
        "abreve;" => "\u{103}",
        "ac;" => "\u{223e}",
        "acE;" => "\u{223e}\u{333}",
        "acd;" => "\u{223f}",
        "acirc" => "\u{e2}",
        "acirc;" => "\u{e2}",
        "acute" => "\u{b4}",
        "acute;" => "\u{b4}",
        "acy;" => "\u{430}",
        "aelig" => "\u{e6}",
        "aelig;" => "\u{e6}",
        "af;" => "\u{2061}",
        "afr;" => "\u{1d51e}",
        "agrave" => "\u{e0}",
        "agrave;" => "\u{e0}",
        "alefsym;" => "\u{2135}",
        "aleph;" => "\u{2135}",
        "alpha;" => "\u{3b1}",
        "amacr;" => "\u{101}",
        "amalg;" => "\u{2a3f}",
        "amp" => "\u{26}",
        "amp;" => "\u{26}",
        "and;" => "\u{2227}",
        "andand;" => "\u{2a55}",
        "andd;" => "\u{2a5c}",
        "andslope;" => "\u{2a58}",
        "andv;" => "\u{2a5a}",
        "ang;" => "\u{2220}",
        "ange;" => "\u{29a4}",
        "angle;" => "\u{2220}",
        "angmsd;" => "\u{2221}",
        "angmsdaa;" => "\u{29a8}",
        "angmsdab;" => "\u{29a9}",
        "angmsdac;" => "\u{29aa}",
        "angmsdad;" => "\u{29ab}",
        "angmsdae;" => "\u{29ac}",
        "angmsdaf;" => "\u{29ad}",
        "angmsdag;" => "\u{29ae}",
        "angmsdah;" => "\u{29af}",
        "angrt;" => "\u{221f}",
        "angrtvb;" => "\u{22be}",
        "angrtvbd;" => "\u{299d}",
        "angsph;" => "\u{2222}",
        "angst;" => "\u{c5}",
        "angzarr;" => "\u{237c}",
        "aogon;" => "\u{105}",
        "aopf;" => "\u{1d552}",
        "ap;" => "\u{2248}",
        "apE;" => "\u{2a70}",
        "apacir;" => "\u{2a6f}",
        "ape;" => "\u{224a}",
        "apid;" => "\u{224b}",
        "apos;" => "\u{27}",
        "approx;" => "\u{2248}",
        "approxeq;" => "\u{224a}",
        "aring" => "\u{e5}",
        "aring;" => "\u{e5}",
        "ascr;" => "\u{1d4b6}",
        "ast;" => "\u{2a}",
        "asymp;" => "\u{2248}",
        "asympeq;" => "\u{224d}",
        "atilde" => "\u{e3}",
        "atilde;" => "\u{e3}",
        "auml" => "\u{e4}",
        "auml;" => "\u{e4}",
        "awconint;" => "\u{2233}",
        "awint;" => "\u{2a11}",
        "bNot;" => "\u{2aed}",
        "backcong;" => "\u{224c}",
        "backepsilon;" => "\u{3f6}",
        "backprime;" => "\u{2035}",
        "backsim;" => "\u{223d}",
        "backsimeq;" => "\u{22cd}",
        "barvee;" => "\u{22bd}",
        "barwed;" => "\u{2305}",
        "barwedge;" => "\u{2305}",
        "bbrk;" => "\u{23b5}",
        "bbrktbrk;" => "\u{23b6}",
        "bcong;" => "\u{224c}",
        "bcy;" => "\u{431}",
        "bdquo;" => "\u{201e}",
        "becaus;" => "\u{2235}",
        "because;" => "\u{2235}",
        "bemptyv;" => "\u{29b0}",
        "bepsi;" => "\u{3f6}",
        "bernou;" => "\u{212c}",
        "beta;" => "\u{3b2}",
        "beth;" => "\u{2136}",
        "between;" => "\u{226c}",
        "bfr;" => "\u{1d51f}",
        "bigcap;" => "\u{22c2}",
        "bigcirc;" => "\u{25ef}",
        "bigcup;" => "\u{22c3}",
        "bigodot;" => "\u{2a00}",
        "bigoplus;" => "\u{2a01}",
        "bigotimes;" => "\u{2a02}",
        "bigsqcup;" => "\u{2a06}",
        "bigstar;" => "\u{2605}",
        "bigtriangledown;" => "\u{25bd}",
        "bigtriangleup;" => "\u{25b3}",
        "biguplus;" => "\u{2a04}",
        "bigvee;" => "\u{22c1}",
        "bigwedge;" => "\u{22c0}",
        "bkarow;" => "\u{290d}",
        "blacklozenge;" => "\u{29eb}",
        "blacksquare;" => "\u{25aa}",
        "blacktriangle;" => "\u{25b4}",
        "blacktriangledown;" => "\u{25be}",
        "blacktriangleleft;" => "\u{25c2}",
        "blacktriangleright;" => "\u{25b8}",
        "blank;" => "\u{2423}",
        "blk12;" => "\u{2592}",
        "blk14;" => "\u{2591}",
        "blk34;" => "\u{2593}",
        "block;" => "\u{2588}",
        "bne;" => "\u{3d}\u{20e5}",
        "bnequiv;" => "\u{2261}\u{20e5}",
        "bnot;" => "\u{2310}",
        "bopf;" => "\u{1d553}",
        "bot;" => "\u{22a5}",
        "bottom;" => "\u{22a5}",
        "bowtie;" => "\u{22c8}",
        "boxDL;" => "\u{2557}",
        "boxDR;" => "\u{2554}",
        "boxDl;" => "\u{2556}",
        "boxDr;" => "\u{2553}",
        "boxH;" => "\u{2550}",
        "boxHD;" => "\u{2566}",
        "boxHU;" => "\u{2569}",
        "boxHd;" => "\u{2564}",
        "boxHu;" => "\u{2567}",
        "boxUL;" => "\u{255d}",
        "boxUR;" => "\u{255a}",
        "boxUl;" => "\u{255c}",
        "boxUr;" => "\u{2559}",
        "boxV;" => "\u{2551}",
        "boxVH;" => "\u{256c}",
        "boxVL;" => "\u{2563}",
        "boxVR;" => "\u{2560}",
        "boxVh;" => "\u{256b}",
        "boxVl;" => "\u{2562}",
        "boxVr;" => "\u{255f}",
        "boxbox;" => "\u{29c9}",
        "boxdL;" => "\u{2555}",
        "boxdR;" => "\u{2552}",
        "boxdl;" => "\u{2510}",
        "boxdr;" => "\u{250c}",
        "boxh;" => "\u{2500}",
        "boxhD;" => "\u{2565}",
        "boxhU;" => "\u{2568}",
        "boxhd;" => "\u{252c}",
        "boxhu;" => "\u{2534}",
        "boxminus;" => "\u{229f}",
        "boxplus;" => "\u{229e}",
        "boxtimes;" => "\u{22a0}",
        "boxuL;" => "\u{255b}",
        "boxuR;" => "\u{2558}",
        "boxul;" => "\u{2518}",
        "boxur;" => "\u{2514}",
        "boxv;" => "\u{2502}",
        "boxvH;" => "\u{256a}",
        "boxvL;" => "\u{2561}",
        "boxvR;" => "\u{255e}",
        "boxvh;" => "\u{253c}",
        "boxvl;" => "\u{2524}",
        "boxvr;" => "\u{251c}",
        "bprime;" => "\u{2035}",
        "breve;" => "\u{2d8}",
        "brvbar" => "\u{a6}",
        "brvbar;" => "\u{a6}",
        "bscr;" => "\u{1d4b7}",
        "bsemi;" => "\u{204f}",
        "bsim;" => "\u{223d}",
        "bsime;" => "\u{22cd}",
        "bsol;" => "\u{5c}",
        "bsolb;" => "\u{29c5}",
        "bsolhsub;" => "\u{27c8}",
        "bull;" => "\u{2022}",
        "bullet;" => "\u{2022}",
        "bump;" => "\u{224e}",
        "bumpE;" => "\u{2aae}",
        "bumpe;" => "\u{224f}",
        "bumpeq;" => "\u{224f}",
        "cacute;" => "\u{107}",
        "cap;" => "\u{2229}",
        "capand;" => "\u{2a44}",
        "capbrcup;" => "\u{2a49}",
        "capcap;" => "\u{2a4b}",
        "capcup;" => "\u{2a47}",
        "capdot;" => "\u{2a40}",
        "caps;" => "\u{2229}\u{fe00}",
        "caret;" => "\u{2041}",
        "caron;" => "\u{2c7}",
        "ccaps;" => "\u{2a4d}",
        "ccaron;" => "\u{10d}",
        "ccedil" => "\u{e7}",
        "ccedil;" => "\u{e7}",
        "ccirc;" => "\u{109}",
        "ccups;" => "\u{2a4c}",
        "ccupssm;" => "\u{2a50}",
        "cdot;" => "\u{10b}",
        "cedil" => "\u{b8}",
        "cedil;" => "\u{b8}",
        "cemptyv;" => "\u{29b2}",
        "cent" => "\u{a2}",
        "cent;" => "\u{a2}",
        "centerdot;" => "\u{b7}",
        "cfr;" => "\u{1d520}",
        "chcy;" => "\u{447}",
        "check;" => "\u{2713}",
        "checkmark;" => "\u{2713}",
        "chi;" => "\u{3c7}",
        "cir;" => "\u{25cb}",
        "cirE;" => "\u{29c3}",
        "circ;" => "\u{2c6}",
        "circeq;" => "\u{2257}",
        "circlearrowleft;" => "\u{21ba}",
        "circlearrowright;" => "\u{21bb}",
        "circledR;" => "\u{ae}",
        "circledS;" => "\u{24c8}",
        "circledast;" => "\u{229b}",
        "circledcirc;" => "\u{229a}",
        "circleddash;" => "\u{229d}",
        "cire;" => "\u{2257}",
        "cirfnint;" => "\u{2a10}",
        "cirmid;" => "\u{2aef}",
        "cirscir;" => "\u{29c2}",
        "clubs;" => "\u{2663}",
        "clubsuit;" => "\u{2663}",
        "colon;" => "\u{3a}",
        "colone;" => "\u{2254}",
        "coloneq;" => "\u{2254}",
        "comma;" => "\u{2c}",
        "commat;" => "\u{40}",
        "comp;" => "\u{2201}",
        "compfn;" => "\u{2218}",
        "complement;" => "\u{2201}",
        "complexes;" => "\u{2102}",
        "cong;" => "\u{2245}",
        "congdot;" => "\u{2a6d}",
        "conint;" => "\u{222e}",
        "copf;" => "\u{1d554}",
        "coprod;" => "\u{2210}",
        "copy" => "\u{a9}",
        "copy;" => "\u{a9}",
        "copysr;" => "\u{2117}",
        "crarr;" => "\u{21b5}",
        "cross;" => "\u{2717}",
        "cscr;" => "\u{1d4b8}",
        "csub;" => "\u{2acf}",
        "csube;" => "\u{2ad1}",
        "csup;" => "\u{2ad0}",
        "csupe;" => "\u{2ad2}",
        "ctdot;" => "\u{22ef}",
        "cudarrl;" => "\u{2938}",
        "cudarrr;" => "\u{2935}",
        "cuepr;" => "\u{22de}",
        "cuesc;" => "\u{22df}",
        "cularr;" => "\u{21b6}",
        "cularrp;" => "\u{293d}",
        "cup;" => "\u{222a}",
        "cupbrcap;" => "\u{2a48}",
        "cupcap;" => "\u{2a46}",
        "cupcup;" => "\u{2a4a}",
        "cupdot;" => "\u{228d}",
        "cupor;" => "\u{2a45}",
        "cups;" => "\u{222a}\u{fe00}",
        "curarr;" => "\u{21b7}",
        "curarrm;" => "\u{293c}",
        "curlyeqprec;" => "\u{22de}",
        "curlyeqsucc;" => "\u{22df}",
        "curlyvee;" => "\u{22ce}",
        "curlywedge;" => "\u{22cf}",
        "curren" => "\u{a4}",
        "curren;" => "\u{a4}",
        "curvearrowleft;" => "\u{21b6}",
        "curvearrowright;" => "\u{21b7}",
        "cuvee;" => "\u{22ce}",
        "cuwed;" => "\u{22cf}",
        "cwconint;" => "\u{2232}",
        "cwint;" => "\u{2231}",
        "cylcty;" => "\u{232d}",
        "dArr;" => "\u{21d3}",
        "dHar;" => "\u{2965}",
        "dagger;" => "\u{2020}",
        "daleth;" => "\u{2138}",
        "darr;" => "\u{2193}",
        "dash;" => "\u{2010}",
        "dashv;" => "\u{22a3}",
        "dbkarow;" => "\u{290f}",
        "dblac;" => "\u{2dd}",
        "dcaron;" => "\u{10f}",
        "dcy;" => "\u{434}",
        "dd;" => "\u{2146}",
        "ddagger;" => "\u{2021}",
        "ddarr;" => "\u{21ca}",
        "ddotseq;" => "\u{2a77}",
        "deg" => "\u{b0}",
        "deg;" => "\u{b0}",
        "delta;" => "\u{3b4}",
        "demptyv;" => "\u{29b1}",
        "dfisht;" => "\u{297f}",
        "dfr;" => "\u{1d521}",
        "dharl;" => "\u{21c3}",
        "dharr;" => "\u{21c2}",
        "diam;" => "\u{22c4}",
        "diamond;" => "\u{22c4}",
        "diamondsuit;" => "\u{2666}",
        "diams;" => "\u{2666}",
        "die;" => "\u{a8}",
        "digamma;" => "\u{3dd}",
        "disin;" => "\u{22f2}",
        "div;" => "\u{f7}",
        "divide" => "\u{f7}",
        "divide;" => "\u{f7}",
        "divideontimes;" => "\u{22c7}",
        "divonx;" => "\u{22c7}",
        "djcy;" => "\u{452}",
        "dlcorn;" => "\u{231e}",
        "dlcrop;" => "\u{230d}",
        "dollar;" => "\u{24}",
        "dopf;" => "\u{1d555}",
        "dot;" => "\u{2d9}",
        "doteq;" => "\u{2250}",
        "doteqdot;" => "\u{2251}",
        "dotminus;" => "\u{2238}",
        "dotplus;" => "\u{2214}",
        "dotsquare;" => "\u{22a1}",
        "doublebarwedge;" => "\u{2306}",
        "downarrow;" => "\u{2193}",
        "downdownarrows;" => "\u{21ca}",
        "downharpoonleft;" => "\u{21c3}",
        "downharpoonright;" => "\u{21c2}",
        "drbkarow;" => "\u{2910}",
        "drcorn;" => "\u{231f}",
        "drcrop;" => "\u{230c}",
        "dscr;" => "\u{1d4b9}",
        "dscy;" => "\u{455}",
        "dsol;" => "\u{29f6}",
        "dstrok;" => "\u{111}",
        "dtdot;" => "\u{22f1}",
        "dtri;" => "\u{25bf}",
        "dtrif;" => "\u{25be}",
        "duarr;" => "\u{21f5}",
        "duhar;" => "\u{296f}",
        "dwangle;" => "\u{29a6}",
        "dzcy;" => "\u{45f}",
        "dzigrarr;" => "\u{27ff}",
        "eDDot;" => "\u{2a77}",
        "eDot;" => "\u{2251}",
        "eacute" => "\u{e9}",
        "eacute;" => "\u{e9}",
        "easter;" => "\u{2a6e}",
        "ecaron;" => "\u{11b}",
        "ecir;" => "\u{2256}",
        "ecirc" => "\u{ea}",
        "ecirc;" => "\u{ea}",
        "ecolon;" => "\u{2255}",
        "ecy;" => "\u{44d}",
        "edot;" => "\u{117}",
        "ee;" => "\u{2147}",
        "efDot;" => "\u{2252}",
        "efr;" => "\u{1d522}",
        "eg;" => "\u{2a9a}",
        "egrave" => "\u{e8}",
        "egrave;" => "\u{e8}",
        "egs;" => "\u{2a96}",
        "egsdot;" => "\u{2a98}",
        "el;" => "\u{2a99}",
        "elinters;" => "\u{23e7}",
        "ell;" => "\u{2113}",
        "els;" => "\u{2a95}",
        "elsdot;" => "\u{2a97}",
        "emacr;" => "\u{113}",
        "empty;" => "\u{2205}",
        "emptyset;" => "\u{2205}",
        "emptyv;" => "\u{2205}",
        "emsp13;" => "\u{2004}",
        "emsp14;" => "\u{2005}",
        "emsp;" => "\u{2003}",
        "eng;" => "\u{14b}",
        "ensp;" => "\u{2002}",
        "eogon;" => "\u{119}",
        "eopf;" => "\u{1d556}",
        "epar;" => "\u{22d5}",
        "eparsl;" => "\u{29e3}",
        "eplus;" => "\u{2a71}",
        "epsi;" => "\u{3b5}",
        "epsilon;" => "\u{3b5}",
        "epsiv;" => "\u{3f5}",
        "eqcirc;" => "\u{2256}",
        "eqcolon;" => "\u{2255}",
        "eqsim;" => "\u{2242}",
        "eqslantgtr;" => "\u{2a96}",
        "eqslantless;" => "\u{2a95}",
        "equals;" => "\u{3d}",
        "equest;" => "\u{225f}",
        "equiv;" => "\u{2261}",
        "equivDD;" => "\u{2a78}",
        "eqvparsl;" => "\u{29e5}",
        "erDot;" => "\u{2253}",
        "erarr;" => "\u{2971}",
        "escr;" => "\u{212f}",
        "esdot;" => "\u{2250}",
        "esim;" => "\u{2242}",
        "eta;" => "\u{3b7}",
        "eth" => "\u{f0}",
        "eth;" => "\u{f0}",
        "euml" => "\u{eb}",
        "euml;" => "\u{eb}",
        "euro;" => "\u{20ac}",
        "excl;" => "\u{21}",
        "exist;" => "\u{2203}",
        "expectation;" => "\u{2130}",
        "exponentiale;" => "\u{2147}",
        "fallingdotseq;" => "\u{2252}",
        "fcy;" => "\u{444}",
        "female;" => "\u{2640}",
        "ffilig;" => "\u{fb03}",
        "fflig;" => "\u{fb00}",
        "ffllig;" => "\u{fb04}",
        "ffr;" => "\u{1d523}",
        "filig;" => "\u{fb01}",
        "fjlig;" => "\u{66}\u{6a}",
        "flat;" => "\u{266d}",
        "fllig;" => "\u{fb02}",
        "fltns;" => "\u{25b1}",
        "fnof;" => "\u{192}",
        "fopf;" => "\u{1d557}",
        "forall;" => "\u{2200}",
        "fork;" => "\u{22d4}",
        "forkv;" => "\u{2ad9}",
        "fpartint;" => "\u{2a0d}",
        "frac12" => "\u{bd}",
        "frac12;" => "\u{bd}",
        "frac13;" => "\u{2153}",
        "frac14" => "\u{bc}",
        "frac14;" => "\u{bc}",
        "frac15;" => "\u{2155}",
        "frac16;" => "\u{2159}",
        "frac18;" => "\u{215b}",
        "frac23;" => "\u{2154}",
        "frac25;" => "\u{2156}",
        "frac34" => "\u{be}",
        "frac34;" => "\u{be}",
        "frac35;" => "\u{2157}",
        "frac38;" => "\u{215c}",
        "frac45;" => "\u{2158}",
        "frac56;" => "\u{215a}",
        "frac58;" => "\u{215d}",
        "frac78;" => "\u{215e}",
        "frasl;" => "\u{2044}",
        "frown;" => "\u{2322}",
        "fscr;" => "\u{1d4bb}",
        "gE;" => "\u{2267}",
        "gEl;" => "\u{2a8c}",
        "gacute;" => "\u{1f5}",
        "gamma;" => "\u{3b3}",
        "gammad;" => "\u{3dd}",
        "gap;" => "\u{2a86}",
        "gbreve;" => "\u{11f}",
        "gcirc;" => "\u{11d}",
        "gcy;" => "\u{433}",
        "gdot;" => "\u{121}",
        "ge;" => "\u{2265}",
        "gel;" => "\u{22db}",
        "geq;" => "\u{2265}",
        "geqq;" => "\u{2267}",
        "geqslant;" => "\u{2a7e}",
        "ges;" => "\u{2a7e}",
        "gescc;" => "\u{2aa9}",
        "gesdot;" => "\u{2a80}",
        "gesdoto;" => "\u{2a82}",
        "gesdotol;" => "\u{2a84}",
        "gesl;" => "\u{22db}\u{fe00}",
        "gesles;" => "\u{2a94}",
        "gfr;" => "\u{1d524}",
        "gg;" => "\u{226b}",
        "ggg;" => "\u{22d9}",
        "gimel;" => "\u{2137}",
        "gjcy;" => "\u{453}",
        "gl;" => "\u{2277}",
        "glE;" => "\u{2a92}",
        "gla;" => "\u{2aa5}",
        "glj;" => "\u{2aa4}",
        "gnE;" => "\u{2269}",
        "gnap;" => "\u{2a8a}",
        "gnapprox;" => "\u{2a8a}",
        "gne;" => "\u{2a88}",
        "gneq;" => "\u{2a88}",
        "gneqq;" => "\u{2269}",
        "gnsim;" => "\u{22e7}",
        "gopf;" => "\u{1d558}",
        "grave;" => "\u{60}",
        "gscr;" => "\u{210a}",
        "gsim;" => "\u{2273}",
        "gsime;" => "\u{2a8e}",
        "gsiml;" => "\u{2a90}",
        "gt" => "\u{3e}",
        "gt;" => "\u{3e}",
        "gtcc;" => "\u{2aa7}",
        "gtcir;" => "\u{2a7a}",
        "gtdot;" => "\u{22d7}",
        "gtlPar;" => "\u{2995}",
        "gtquest;" => "\u{2a7c}",
        "gtrapprox;" => "\u{2a86}",
        "gtrarr;" => "\u{2978}",
        "gtrdot;" => "\u{22d7}",
        "gtreqless;" => "\u{22db}",
        "gtreqqless;" => "\u{2a8c}",
        "gtrless;" => "\u{2277}",
        "gtrsim;" => "\u{2273}",
        "gvertneqq;" => "\u{2269}\u{fe00}",
        "gvnE;" => "\u{2269}\u{fe00}",
        "hArr;" => "\u{21d4}",
        "hairsp;" => "\u{200a}",
        "half;" => "\u{bd}",
        "hamilt;" => "\u{210b}",
        "hardcy;" => "\u{44a}",
        "harr;" => "\u{2194}",
        "harrcir;" => "\u{2948}",
        "harrw;" => "\u{21ad}",
        "hbar;" => "\u{210f}",
        "hcirc;" => "\u{125}",
        "hearts;" => "\u{2665}",
        "heartsuit;" => "\u{2665}",
        "hellip;" => "\u{2026}",
        "hercon;" => "\u{22b9}",
        "hfr;" => "\u{1d525}",
        "hksearow;" => "\u{2925}",
        "hkswarow;" => "\u{2926}",
        "hoarr;" => "\u{21ff}",
        "homtht;" => "\u{223b}",
        "hookleftarrow;" => "\u{21a9}",
        "hookrightarrow;" => "\u{21aa}",
        "hopf;" => "\u{1d559}",
        "horbar;" => "\u{2015}",
        "hscr;" => "\u{1d4bd}",
        "hslash;" => "\u{210f}",
        "hstrok;" => "\u{127}",
        "hybull;" => "\u{2043}",
        "hyphen;" => "\u{2010}",
        "iacute" => "\u{ed}",
        "iacute;" => "\u{ed}",
        "ic;" => "\u{2063}",
        "icirc" => "\u{ee}",
        "icirc;" => "\u{ee}",
        "icy;" => "\u{438}",
        "iecy;" => "\u{435}",
        "iexcl" => "\u{a1}",
        "iexcl;" => "\u{a1}",
        "iff;" => "\u{21d4}",
        "ifr;" => "\u{1d526}",
        "igrave" => "\u{ec}",
        "igrave;" => "\u{ec}",
        "ii;" => "\u{2148}",
        "iiiint;" => "\u{2a0c}",
        "iiint;" => "\u{222d}",
        "iinfin;" => "\u{29dc}",
        "iiota;" => "\u{2129}",
        "ijlig;" => "\u{133}",
        "imacr;" => "\u{12b}",
        "image;" => "\u{2111}",
        "imagline;" => "\u{2110}",
        "imagpart;" => "\u{2111}",
        "imath;" => "\u{131}",
        "imof;" => "\u{22b7}",
        "imped;" => "\u{1b5}",
        "in;" => "\u{2208}",
        "incare;" => "\u{2105}",
        "infin;" => "\u{221e}",
        "infintie;" => "\u{29dd}",
        "inodot;" => "\u{131}",
        "int;" => "\u{222b}",
        "intcal;" => "\u{22ba}",
        "integers;" => "\u{2124}",
        "intercal;" => "\u{22ba}",
        "intlarhk;" => "\u{2a17}",
        "intprod;" => "\u{2a3c}",
        "iocy;" => "\u{451}",
        "iogon;" => "\u{12f}",
        "iopf;" => "\u{1d55a}",
        "iota;" => "\u{3b9}",
        "iprod;" => "\u{2a3c}",
        "iquest" => "\u{bf}",
        "iquest;" => "\u{bf}",
        "iscr;" => "\u{1d4be}",
        "isin;" => "\u{2208}",
        "isinE;" => "\u{22f9}",
        "isindot;" => "\u{22f5}",
        "isins;" => "\u{22f4}",
        "isinsv;" => "\u{22f3}",
        "isinv;" => "\u{2208}",
        "it;" => "\u{2062}",
        "itilde;" => "\u{129}",
        "iukcy;" => "\u{456}",
        "iuml" => "\u{ef}",
        "iuml;" => "\u{ef}",
        "jcirc;" => "\u{135}",
        "jcy;" => "\u{439}",
        "jfr;" => "\u{1d527}",
        "jmath;" => "\u{237}",
        "jopf;" => "\u{1d55b}",
        "jscr;" => "\u{1d4bf}",
        "jsercy;" => "\u{458}",
        "jukcy;" => "\u{454}",
        "kappa;" => "\u{3ba}",
        "kappav;" => "\u{3f0}",
        "kcedil;" => "\u{137}",
        "kcy;" => "\u{43a}",
        "kfr;" => "\u{1d528}",
        "kgreen;" => "\u{138}",
        "khcy;" => "\u{445}",
        "kjcy;" => "\u{45c}",
        "kopf;" => "\u{1d55c}",
        "kscr;" => "\u{1d4c0}",
        "lAarr;" => "\u{21da}",
        "lArr;" => "\u{21d0}",
        "lAtail;" => "\u{291b}",
        "lBarr;" => "\u{290e}",
        "lE;" => "\u{2266}",
        "lEg;" => "\u{2a8b}",
        "lHar;" => "\u{2962}",
        "lacute;" => "\u{13a}",
        "laemptyv;" => "\u{29b4}",
        "lagran;" => "\u{2112}",
        "lambda;" => "\u{3bb}",
        "lang;" => "\u{27e8}",
        "langd;" => "\u{2991}",
        "langle;" => "\u{27e8}",
        "lap;" => "\u{2a85}",
        "laquo" => "\u{ab}",
        "laquo;" => "\u{ab}",
        "larr;" => "\u{2190}",
        "larrb;" => "\u{21e4}",
        "larrbfs;" => "\u{291f}",
        "larrfs;" => "\u{291d}",
        "larrhk;" => "\u{21a9}",
        "larrlp;" => "\u{21ab}",
        "larrpl;" => "\u{2939}",
        "larrsim;" => "\u{2973}",
        "larrtl;" => "\u{21a2}",
        "lat;" => "\u{2aab}",
        "latail;" => "\u{2919}",
        "late;" => "\u{2aad}",
        "lates;" => "\u{2aad}\u{fe00}",
        "lbarr;" => "\u{290c}",
        "lbbrk;" => "\u{2772}",
        "lbrace;" => "\u{7b}",
        "lbrack;" => "\u{5b}",
        "lbrke;" => "\u{298b}",
        "lbrksld;" => "\u{298f}",
        "lbrkslu;" => "\u{298d}",
        "lcaron;" => "\u{13e}",
        "lcedil;" => "\u{13c}",
        "lceil;" => "\u{2308}",
        "lcub;" => "\u{7b}",
        "lcy;" => "\u{43b}",
        "ldca;" => "\u{2936}",
        "ldquo;" => "\u{201c}",
        "ldquor;" => "\u{201e}",
        "ldrdhar;" => "\u{2967}",
        "ldrushar;" => "\u{294b}",
        "ldsh;" => "\u{21b2}",
        "le;" => "\u{2264}",
        "leftarrow;" => "\u{2190}",
        "leftarrowtail;" => "\u{21a2}",
        "leftharpoondown;" => "\u{21bd}",
        "leftharpoonup;" => "\u{21bc}",
        "leftleftarrows;" => "\u{21c7}",
        "leftrightarrow;" => "\u{2194}",
        "leftrightarrows;" => "\u{21c6}",
        "leftrightharpoons;" => "\u{21cb}",
        "leftrightsquigarrow;" => "\u{21ad}",
        "leftthreetimes;" => "\u{22cb}",
        "leg;" => "\u{22da}",
        "leq;" => "\u{2264}",
        "leqq;" => "\u{2266}",
        "leqslant;" => "\u{2a7d}",
        "les;" => "\u{2a7d}",
        "lescc;" => "\u{2aa8}",
        "lesdot;" => "\u{2a7f}",
        "lesdoto;" => "\u{2a81}",
        "lesdotor;" => "\u{2a83}",
        "lesg;" => "\u{22da}\u{fe00}",
        "lesges;" => "\u{2a93}",
        "lessapprox;" => "\u{2a85}",
        "lessdot;" => "\u{22d6}",
        "lesseqgtr;" => "\u{22da}",
        "lesseqqgtr;" => "\u{2a8b}",
        "lessgtr;" => "\u{2276}",
        "lesssim;" => "\u{2272}",
        "lfisht;" => "\u{297c}",
        "lfloor;" => "\u{230a}",
        "lfr;" => "\u{1d529}",
        "lg;" => "\u{2276}",
        "lgE;" => "\u{2a91}",
        "lhard;" => "\u{21bd}",
        "lharu;" => "\u{21bc}",
        "lharul;" => "\u{296a}",
        "lhblk;" => "\u{2584}",
        "ljcy;" => "\u{459}",
        "ll;" => "\u{226a}",
        "llarr;" => "\u{21c7}",
        "llcorner;" => "\u{231e}",
        "llhard;" => "\u{296b}",
        "lltri;" => "\u{25fa}",
        "lmidot;" => "\u{140}",
        "lmoust;" => "\u{23b0}",
        "lmoustache;" => "\u{23b0}",
        "lnE;" => "\u{2268}",
        "lnap;" => "\u{2a89}",
        "lnapprox;" => "\u{2a89}",
        "lne;" => "\u{2a87}",
        "lneq;" => "\u{2a87}",
        "lneqq;" => "\u{2268}",
        "lnsim;" => "\u{22e6}",
        "loang;" => "\u{27ec}",
        "loarr;" => "\u{21fd}",
        "lobrk;" => "\u{27e6}",
        "longleftarrow;" => "\u{27f5}",
        "longleftrightarrow;" => "\u{27f7}",
        "longmapsto;" => "\u{27fc}",
        "longrightarrow;" => "\u{27f6}",
        "looparrowleft;" => "\u{21ab}",
        "looparrowright;" => "\u{21ac}",
        "lopar;" => "\u{2985}",
        "lopf;" => "\u{1d55d}",
        "loplus;" => "\u{2a2d}",
        "lotimes;" => "\u{2a34}",
        "lowast;" => "\u{2217}",
        "lowbar;" => "\u{5f}",
        "loz;" => "\u{25ca}",
        "lozenge;" => "\u{25ca}",
        "lozf;" => "\u{29eb}",
        "lpar;" => "\u{28}",
        "lparlt;" => "\u{2993}",
        "lrarr;" => "\u{21c6}",
        "lrcorner;" => "\u{231f}",
        "lrhar;" => "\u{21cb}",
        "lrhard;" => "\u{296d}",
        "lrm;" => "\u{200e}",
        "lrtri;" => "\u{22bf}",
        "lsaquo;" => "\u{2039}",
        "lscr;" => "\u{1d4c1}",
        "lsh;" => "\u{21b0}",
        "lsim;" => "\u{2272}",
        "lsime;" => "\u{2a8d}",
        "lsimg;" => "\u{2a8f}",
        "lsqb;" => "\u{5b}",
        "lsquo;" => "\u{2018}",
        "lsquor;" => "\u{201a}",
        "lstrok;" => "\u{142}",
        "lt" => "\u{3c}",
        "lt;" => "\u{3c}",
        "ltcc;" => "\u{2aa6}",
        "ltcir;" => "\u{2a79}",
        "ltdot;" => "\u{22d6}",
        "lthree;" => "\u{22cb}",
        "ltimes;" => "\u{22c9}",
        "ltlarr;" => "\u{2976}",
        "ltquest;" => "\u{2a7b}",
        "ltrPar;" => "\u{2996}",
        "ltri;" => "\u{25c3}",
        "ltrie;" => "\u{22b4}",
        "ltrif;" => "\u{25c2}",
        "lurdshar;" => "\u{294a}",
        "luruhar;" => "\u{2966}",
        "lvertneqq;" => "\u{2268}\u{fe00}",
        "lvnE;" => "\u{2268}\u{fe00}",
        "mDDot;" => "\u{223a}",
        "macr" => "\u{af}",
        "macr;" => "\u{af}",
        "male;" => "\u{2642}",
        "malt;" => "\u{2720}",
        "maltese;" => "\u{2720}",
        "map;" => "\u{21a6}",
        "mapsto;" => "\u{21a6}",
        "mapstodown;" => "\u{21a7}",
        "mapstoleft;" => "\u{21a4}",
        "mapstoup;" => "\u{21a5}",
        "marker;" => "\u{25ae}",
        "mcomma;" => "\u{2a29}",
        "mcy;" => "\u{43c}",
        "mdash;" => "\u{2014}",
        "measuredangle;" => "\u{2221}",
        "mfr;" => "\u{1d52a}",
        "mho;" => "\u{2127}",
        "micro" => "\u{b5}",
        "micro;" => "\u{b5}",
        "mid;" => "\u{2223}",
        "midast;" => "\u{2a}",
        "midcir;" => "\u{2af0}",
        "middot" => "\u{b7}",
        "middot;" => "\u{b7}",
        "minus;" => "\u{2212}",
        "minusb;" => "\u{229f}",
        "minusd;" => "\u{2238}",
        "minusdu;" => "\u{2a2a}",
        "mlcp;" => "\u{2adb}",
        "mldr;" => "\u{2026}",
        "mnplus;" => "\u{2213}",
        "models;" => "\u{22a7}",
        "mopf;" => "\u{1d55e}",
        "mp;" => "\u{2213}",
        "mscr;" => "\u{1d4c2}",
        "mstpos;" => "\u{223e}",
        "mu;" => "\u{3bc}",
        "multimap;" => "\u{22b8}",
        "mumap;" => "\u{22b8}",
        "nGg;" => "\u{22d9}\u{338}",
        "nGt;" => "\u{226b}\u{20d2}",
        "nGtv;" => "\u{226b}\u{338}",
        "nLeftarrow;" => "\u{21cd}",
        "nLeftrightarrow;" => "\u{21ce}",
        "nLl;" => "\u{22d8}\u{338}",
        "nLt;" => "\u{226a}\u{20d2}",
        "nLtv;" => "\u{226a}\u{338}",
        "nRightarrow;" => "\u{21cf}",
        "nVDash;" => "\u{22af}",
        "nVdash;" => "\u{22ae}",
        "nabla;" => "\u{2207}",
        "nacute;" => "\u{144}",
        "nang;" => "\u{2220}\u{20d2}",
        "nap;" => "\u{2249}",
        "napE;" => "\u{2a70}\u{338}",
        "napid;" => "\u{224b}\u{338}",
        "napos;" => "\u{149}",
        "napprox;" => "\u{2249}",
        "natur;" => "\u{266e}",
        "natural;" => "\u{266e}",
        "naturals;" => "\u{2115}",
        "nbsp" => "\u{a0}",
        "nbsp;" => "\u{a0}",
        "nbump;" => "\u{224e}\u{338}",
        "nbumpe;" => "\u{224f}\u{338}",
        "ncap;" => "\u{2a43}",
        "ncaron;" => "\u{148}",
        "ncedil;" => "\u{146}",
        "ncong;" => "\u{2247}",
        "ncongdot;" => "\u{2a6d}\u{338}",
        "ncup;" => "\u{2a42}",
        "ncy;" => "\u{43d}",
        "ndash;" => "\u{2013}",
        "ne;" => "\u{2260}",
        "neArr;" => "\u{21d7}",
        "nearhk;" => "\u{2924}",
        "nearr;" => "\u{2197}",
        "nearrow;" => "\u{2197}",
        "nedot;" => "\u{2250}\u{338}",
        "nequiv;" => "\u{2262}",
        "nesear;" => "\u{2928}",
        "nesim;" => "\u{2242}\u{338}",
        "nexist;" => "\u{2204}",
        "nexists;" => "\u{2204}",
        "nfr;" => "\u{1d52b}",
        "ngE;" => "\u{2267}\u{338}",
        "nge;" => "\u{2271}",
        "ngeq;" => "\u{2271}",
        "ngeqq;" => "\u{2267}\u{338}",
        "ngeqslant;" => "\u{2a7e}\u{338}",
        "nges;" => "\u{2a7e}\u{338}",
        "ngsim;" => "\u{2275}",
        "ngt;" => "\u{226f}",
        "ngtr;" => "\u{226f}",
        "nhArr;" => "\u{21ce}",
        "nharr;" => "\u{21ae}",
        "nhpar;" => "\u{2af2}",
        "ni;" => "\u{220b}",
        "nis;" => "\u{22fc}",
        "nisd;" => "\u{22fa}",
        "niv;" => "\u{220b}",
        "njcy;" => "\u{45a}",
        "nlArr;" => "\u{21cd}",
        "nlE;" => "\u{2266}\u{338}",
        "nlarr;" => "\u{219a}",
        "nldr;" => "\u{2025}",
        "nle;" => "\u{2270}",
        "nleftarrow;" => "\u{219a}",
        "nleftrightarrow;" => "\u{21ae}",
        "nleq;" => "\u{2270}",
        "nleqq;" => "\u{2266}\u{338}",
        "nleqslant;" => "\u{2a7d}\u{338}",
        "nles;" => "\u{2a7d}\u{338}",
        "nless;" => "\u{226e}",
        "nlsim;" => "\u{2274}",
        "nlt;" => "\u{226e}",
        "nltri;" => "\u{22ea}",
        "nltrie;" => "\u{22ec}",
        "nmid;" => "\u{2224}",
        "nopf;" => "\u{1d55f}",
        "not" => "\u{ac}",
        "not;" => "\u{ac}",
        "notin;" => "\u{2209}",
        "notinE;" => "\u{22f9}\u{338}",
        "notindot;" => "\u{22f5}\u{338}",
        "notinva;" => "\u{2209}",
        "notinvb;" => "\u{22f7}",
        "notinvc;" => "\u{22f6}",
        "notni;" => "\u{220c}",
        "notniva;" => "\u{220c}",
        "notnivb;" => "\u{22fe}",
        "notnivc;" => "\u{22fd}",
        "npar;" => "\u{2226}",
        "nparallel;" => "\u{2226}",
        "nparsl;" => "\u{2afd}\u{20e5}",
        "npart;" => "\u{2202}\u{338}",
        "npolint;" => "\u{2a14}",
        "npr;" => "\u{2280}",
        "nprcue;" => "\u{22e0}",
        "npre;" => "\u{2aaf}\u{338}",
        "nprec;" => "\u{2280}",
        "npreceq;" => "\u{2aaf}\u{338}",
        "nrArr;" => "\u{21cf}",
        "nrarr;" => "\u{219b}",
        "nrarrc;" => "\u{2933}\u{338}",
        "nrarrw;" => "\u{219d}\u{338}",
        "nrightarrow;" => "\u{219b}",
        "nrtri;" => "\u{22eb}",
        "nrtrie;" => "\u{22ed}",
        "nsc;" => "\u{2281}",
        "nsccue;" => "\u{22e1}",
        "nsce;" => "\u{2ab0}\u{338}",
        "nscr;" => "\u{1d4c3}",
        "nshortmid;" => "\u{2224}",
        "nshortparallel;" => "\u{2226}",
        "nsim;" => "\u{2241}",
        "nsime;" => "\u{2244}",
        "nsimeq;" => "\u{2244}",
        "nsmid;" => "\u{2224}",
        "nspar;" => "\u{2226}",
        "nsqsube;" => "\u{22e2}",
        "nsqsupe;" => "\u{22e3}",
        "nsub;" => "\u{2284}",
        "nsubE;" => "\u{2ac5}\u{338}",
        "nsube;" => "\u{2288}",
        "nsubset;" => "\u{2282}\u{20d2}",
        "nsubseteq;" => "\u{2288}",
        "nsubseteqq;" => "\u{2ac5}\u{338}",
        "nsucc;" => "\u{2281}",
        "nsucceq;" => "\u{2ab0}\u{338}",
        "nsup;" => "\u{2285}",
        "nsupE;" => "\u{2ac6}\u{338}",
        "nsupe;" => "\u{2289}",
        "nsupset;" => "\u{2283}\u{20d2}",
        "nsupseteq;" => "\u{2289}",
        "nsupseteqq;" => "\u{2ac6}\u{338}",
        "ntgl;" => "\u{2279}",
        "ntilde" => "\u{f1}",
        "ntilde;" => "\u{f1}",
        "ntlg;" => "\u{2278}",
        "ntriangleleft;" => "\u{22ea}",
        "ntrianglelefteq;" => "\u{22ec}",
        "ntriangleright;" => "\u{22eb}",
        "ntrianglerighteq;" => "\u{22ed}",
        "nu;" => "\u{3bd}",
        "num;" => "\u{23}",
        "numero;" => "\u{2116}",
        "numsp;" => "\u{2007}",
        "nvDash;" => "\u{22ad}",
        "nvHarr;" => "\u{2904}",
        "nvap;" => "\u{224d}\u{20d2}",
        "nvdash;" => "\u{22ac}",
        "nvge;" => "\u{2265}\u{20d2}",
        "nvgt;" => "\u{3e}\u{20d2}",
        "nvinfin;" => "\u{29de}",
        "nvlArr;" => "\u{2902}",
        "nvle;" => "\u{2264}\u{20d2}",
        "nvlt;" => "\u{3c}\u{20d2}",
        "nvltrie;" => "\u{22b4}\u{20d2}",
        "nvrArr;" => "\u{2903}",
        "nvrtrie;" => "\u{22b5}\u{20d2}",
        "nvsim;" => "\u{223c}\u{20d2}",
        "nwArr;" => "\u{21d6}",
        "nwarhk;" => "\u{2923}",
        "nwarr;" => "\u{2196}",
        "nwarrow;" => "\u{2196}",
        "nwnear;" => "\u{2927}",
        "oS;" => "\u{24c8}",
        "oacute" => "\u{f3}",
        "oacute;" => "\u{f3}",
        "oast;" => "\u{229b}",
        "ocir;" => "\u{229a}",
        "ocirc" => "\u{f4}",
        "ocirc;" => "\u{f4}",
        "ocy;" => "\u{43e}",
        "odash;" => "\u{229d}",
        "odblac;" => "\u{151}",
        "odiv;" => "\u{2a38}",
        "odot;" => "\u{2299}",
        "odsold;" => "\u{29bc}",
        "oelig;" => "\u{153}",
        "ofcir;" => "\u{29bf}",
        "ofr;" => "\u{1d52c}",
        "ogon;" => "\u{2db}",
        "ograve" => "\u{f2}",
        "ograve;" => "\u{f2}",
        "ogt;" => "\u{29c1}",
        "ohbar;" => "\u{29b5}",
        "ohm;" => "\u{3a9}",
        "oint;" => "\u{222e}",
        "olarr;" => "\u{21ba}",
        "olcir;" => "\u{29be}",
        "olcross;" => "\u{29bb}",
        "oline;" => "\u{203e}",
        "olt;" => "\u{29c0}",
        "omacr;" => "\u{14d}",
        "omega;" => "\u{3c9}",
        "omicron;" => "\u{3bf}",
        "omid;" => "\u{29b6}",
        "ominus;" => "\u{2296}",
        "oopf;" => "\u{1d560}",
        "opar;" => "\u{29b7}",
        "operp;" => "\u{29b9}",
        "oplus;" => "\u{2295}",
        "or;" => "\u{2228}",
        "orarr;" => "\u{21bb}",
        "ord;" => "\u{2a5d}",
        "order;" => "\u{2134}",
        "orderof;" => "\u{2134}",
        "ordf" => "\u{aa}",
        "ordf;" => "\u{aa}",
        "ordm" => "\u{ba}",
        "ordm;" => "\u{ba}",
        "origof;" => "\u{22b6}",
        "oror;" => "\u{2a56}",
        "orslope;" => "\u{2a57}",
        "orv;" => "\u{2a5b}",
        "oscr;" => "\u{2134}",
        "oslash" => "\u{f8}",
        "oslash;" => "\u{f8}",
        "osol;" => "\u{2298}",
        "otilde" => "\u{f5}",
        "otilde;" => "\u{f5}",
        "otimes;" => "\u{2297}",
        "otimesas;" => "\u{2a36}",
        "ouml" => "\u{f6}",
        "ouml;" => "\u{f6}",
        "ovbar;" => "\u{233d}",
        "par;" => "\u{2225}",
        "para" => "\u{b6}",
        "para;" => "\u{b6}",
        "parallel;" => "\u{2225}",
        "parsim;" => "\u{2af3}",
        "parsl;" => "\u{2afd}",
        "part;" => "\u{2202}",
        "pcy;" => "\u{43f}",
        "percnt;" => "\u{25}",
        "period;" => "\u{2e}",
        "permil;" => "\u{2030}",
        "perp;" => "\u{22a5}",
        "pertenk;" => "\u{2031}",
        "pfr;" => "\u{1d52d}",
        "phi;" => "\u{3c6}",
        "phiv;" => "\u{3d5}",
        "phmmat;" => "\u{2133}",
        "phone;" => "\u{260e}",
        "pi;" => "\u{3c0}",
        "pitchfork;" => "\u{22d4}",
        "piv;" => "\u{3d6}",
        "planck;" => "\u{210f}",
        "planckh;" => "\u{210e}",
        "plankv;" => "\u{210f}",
        "plus;" => "\u{2b}",
        "plusacir;" => "\u{2a23}",
        "plusb;" => "\u{229e}",
        "pluscir;" => "\u{2a22}",
        "plusdo;" => "\u{2214}",
        "plusdu;" => "\u{2a25}",
        "pluse;" => "\u{2a72}",
        "plusmn" => "\u{b1}",
        "plusmn;" => "\u{b1}",
        "plussim;" => "\u{2a26}",
        "plustwo;" => "\u{2a27}",
        "pm;" => "\u{b1}",
        "pointint;" => "\u{2a15}",
        "popf;" => "\u{1d561}",
        "pound" => "\u{a3}",
        "pound;" => "\u{a3}",
        "pr;" => "\u{227a}",
        "prE;" => "\u{2ab3}",
        "prap;" => "\u{2ab7}",
        "prcue;" => "\u{227c}",
        "pre;" => "\u{2aaf}",
        "prec;" => "\u{227a}",
        "precapprox;" => "\u{2ab7}",
        "preccurlyeq;" => "\u{227c}",
        "preceq;" => "\u{2aaf}",
        "precnapprox;" => "\u{2ab9}",
        "precneqq;" => "\u{2ab5}",
        "precnsim;" => "\u{22e8}",
        "precsim;" => "\u{227e}",
        "prime;" => "\u{2032}",
        "primes;" => "\u{2119}",
        "prnE;" => "\u{2ab5}",
        "prnap;" => "\u{2ab9}",
        "prnsim;" => "\u{22e8}",
        "prod;" => "\u{220f}",
        "profalar;" => "\u{232e}",
        "profline;" => "\u{2312}",
        "profsurf;" => "\u{2313}",
        "prop;" => "\u{221d}",
        "propto;" => "\u{221d}",
        "prsim;" => "\u{227e}",
        "prurel;" => "\u{22b0}",
        "pscr;" => "\u{1d4c5}",
        "psi;" => "\u{3c8}",
        "puncsp;" => "\u{2008}",
        "qfr;" => "\u{1d52e}",
        "qint;" => "\u{2a0c}",
        "qopf;" => "\u{1d562}",
        "qprime;" => "\u{2057}",
        "qscr;" => "\u{1d4c6}",
        "quaternions;" => "\u{210d}",
        "quatint;" => "\u{2a16}",
        "quest;" => "\u{3f}",
        "questeq;" => "\u{225f}",
        "quot" => "\u{22}",
        "quot;" => "\u{22}",
        "rAarr;" => "\u{21db}",
        "rArr;" => "\u{21d2}",
        "rAtail;" => "\u{291c}",
        "rBarr;" => "\u{290f}",
        "rHar;" => "\u{2964}",
        "race;" => "\u{223d}\u{331}",
        "racute;" => "\u{155}",
        "radic;" => "\u{221a}",
        "raemptyv;" => "\u{29b3}",
        "rang;" => "\u{27e9}",
        "rangd;" => "\u{2992}",
        "range;" => "\u{29a5}",
        "rangle;" => "\u{27e9}",
        "raquo" => "\u{bb}",
        "raquo;" => "\u{bb}",
        "rarr;" => "\u{2192}",
        "rarrap;" => "\u{2975}",
        "rarrb;" => "\u{21e5}",
        "rarrbfs;" => "\u{2920}",
        "rarrc;" => "\u{2933}",
        "rarrfs;" => "\u{291e}",
        "rarrhk;" => "\u{21aa}",
        "rarrlp;" => "\u{21ac}",
        "rarrpl;" => "\u{2945}",
        "rarrsim;" => "\u{2974}",
        "rarrtl;" => "\u{21a3}",
        "rarrw;" => "\u{219d}",
        "ratail;" => "\u{291a}",
        "ratio;" => "\u{2236}",
        "rationals;" => "\u{211a}",
        "rbarr;" => "\u{290d}",
        "rbbrk;" => "\u{2773}",
        "rbrace;" => "\u{7d}",
        "rbrack;" => "\u{5d}",
        "rbrke;" => "\u{298c}",
        "rbrksld;" => "\u{298e}",
        "rbrkslu;" => "\u{2990}",
        "rcaron;" => "\u{159}",
        "rcedil;" => "\u{157}",
        "rceil;" => "\u{2309}",
        "rcub;" => "\u{7d}",
        "rcy;" => "\u{440}",
        "rdca;" => "\u{2937}",
        "rdldhar;" => "\u{2969}",
        "rdquo;" => "\u{201d}",
        "rdquor;" => "\u{201d}",
        "rdsh;" => "\u{21b3}",
        "real;" => "\u{211c}",
        "realine;" => "\u{211b}",
        "realpart;" => "\u{211c}",
        "reals;" => "\u{211d}",
        "rect;" => "\u{25ad}",
        "reg" => "\u{ae}",
        "reg;" => "\u{ae}",
        "rfisht;" => "\u{297d}",
        "rfloor;" => "\u{230b}",
        "rfr;" => "\u{1d52f}",
        "rhard;" => "\u{21c1}",
        "rharu;" => "\u{21c0}",
        "rharul;" => "\u{296c}",
        "rho;" => "\u{3c1}",
        "rhov;" => "\u{3f1}",
        "rightarrow;" => "\u{2192}",
        "rightarrowtail;" => "\u{21a3}",
        "rightharpoondown;" => "\u{21c1}",
        "rightharpoonup;" => "\u{21c0}",
        "rightleftarrows;" => "\u{21c4}",
        "rightleftharpoons;" => "\u{21cc}",
        "rightrightarrows;" => "\u{21c9}",
        "rightsquigarrow;" => "\u{219d}",
        "rightthreetimes;" => "\u{22cc}",
        "ring;" => "\u{2da}",
        "risingdotseq;" => "\u{2253}",
        "rlarr;" => "\u{21c4}",
        "rlhar;" => "\u{21cc}",
        "rlm;" => "\u{200f}",
        "rmoust;" => "\u{23b1}",
        "rmoustache;" => "\u{23b1}",
        "rnmid;" => "\u{2aee}",
        "roang;" => "\u{27ed}",
        "roarr;" => "\u{21fe}",
        "robrk;" => "\u{27e7}",
        "ropar;" => "\u{2986}",
        "ropf;" => "\u{1d563}",
        "roplus;" => "\u{2a2e}",
        "rotimes;" => "\u{2a35}",
        "rpar;" => "\u{29}",
        "rpargt;" => "\u{2994}",
        "rppolint;" => "\u{2a12}",
        "rrarr;" => "\u{21c9}",
        "rsaquo;" => "\u{203a}",
        "rscr;" => "\u{1d4c7}",
        "rsh;" => "\u{21b1}",
        "rsqb;" => "\u{5d}",
        "rsquo;" => "\u{2019}",
        "rsquor;" => "\u{2019}",
        "rthree;" => "\u{22cc}",
        "rtimes;" => "\u{22ca}",
        "rtri;" => "\u{25b9}",
        "rtrie;" => "\u{22b5}",
        "rtrif;" => "\u{25b8}",
        "rtriltri;" => "\u{29ce}",
        "ruluhar;" => "\u{2968}",
        "rx;" => "\u{211e}",
        "sacute;" => "\u{15b}",
        "sbquo;" => "\u{201a}",
        "sc;" => "\u{227b}",
        "scE;" => "\u{2ab4}",
        "scap;" => "\u{2ab8}",
        "scaron;" => "\u{161}",
        "sccue;" => "\u{227d}",
        "sce;" => "\u{2ab0}",
        "scedil;" => "\u{15f}",
        "scirc;" => "\u{15d}",
        "scnE;" => "\u{2ab6}",
        "scnap;" => "\u{2aba}",
        "scnsim;" => "\u{22e9}",
        "scpolint;" => "\u{2a13}",
        "scsim;" => "\u{227f}",
        "scy;" => "\u{441}",
        "sdot;" => "\u{22c5}",
        "sdotb;" => "\u{22a1}",
        "sdote;" => "\u{2a66}",
        "seArr;" => "\u{21d8}",
        "searhk;" => "\u{2925}",
        "searr;" => "\u{2198}",
        "searrow;" => "\u{2198}",
        "sect" => "\u{a7}",
        "sect;" => "\u{a7}",
        "semi;" => "\u{3b}",
        "seswar;" => "\u{2929}",
        "setminus;" => "\u{2216}",
        "setmn;" => "\u{2216}",
        "sext;" => "\u{2736}",
        "sfr;" => "\u{1d530}",
        "sfrown;" => "\u{2322}",
        "sharp;" => "\u{266f}",
        "shchcy;" => "\u{449}",
        "shcy;" => "\u{448}",
        "shortmid;" => "\u{2223}",
        "shortparallel;" => "\u{2225}",
        "shy" => "\u{ad}",
        "shy;" => "\u{ad}",
        "sigma;" => "\u{3c3}",
        "sigmaf;" => "\u{3c2}",
        "sigmav;" => "\u{3c2}",
        "sim;" => "\u{223c}",
        "simdot;" => "\u{2a6a}",
        "sime;" => "\u{2243}",
        "simeq;" => "\u{2243}",
        "simg;" => "\u{2a9e}",
        "simgE;" => "\u{2aa0}",
        "siml;" => "\u{2a9d}",
        "simlE;" => "\u{2a9f}",
        "simne;" => "\u{2246}",
        "simplus;" => "\u{2a24}",
        "simrarr;" => "\u{2972}",
        "slarr;" => "\u{2190}",
        "smallsetminus;" => "\u{2216}",
        "smashp;" => "\u{2a33}",
        "smeparsl;" => "\u{29e4}",
        "smid;" => "\u{2223}",
        "smile;" => "\u{2323}",
        "smt;" => "\u{2aaa}",
        "smte;" => "\u{2aac}",
        "smtes;" => "\u{2aac}\u{fe00}",
        "softcy;" => "\u{44c}",
        "sol;" => "\u{2f}",
        "solb;" => "\u{29c4}",
        "solbar;" => "\u{233f}",
        "sopf;" => "\u{1d564}",
        "spades;" => "\u{2660}",
        "spadesuit;" => "\u{2660}",
        "spar;" => "\u{2225}",
        "sqcap;" => "\u{2293}",
        "sqcaps;" => "\u{2293}\u{fe00}",
        "sqcup;" => "\u{2294}",
        "sqcups;" => "\u{2294}\u{fe00}",
        "sqsub;" => "\u{228f}",
        "sqsube;" => "\u{2291}",
        "sqsubset;" => "\u{228f}",
        "sqsubseteq;" => "\u{2291}",
        "sqsup;" => "\u{2290}",
        "sqsupe;" => "\u{2292}",
        "sqsupset;" => "\u{2290}",
        "sqsupseteq;" => "\u{2292}",
        "squ;" => "\u{25a1}",
        "square;" => "\u{25a1}",
        "squarf;" => "\u{25aa}",
        "squf;" => "\u{25aa}",
        "srarr;" => "\u{2192}",
        "sscr;" => "\u{1d4c8}",
        "ssetmn;" => "\u{2216}",
        "ssmile;" => "\u{2323}",
        "sstarf;" => "\u{22c6}",
        "star;" => "\u{2606}",
        "starf;" => "\u{2605}",
        "straightepsilon;" => "\u{3f5}",
        "straightphi;" => "\u{3d5}",
        "strns;" => "\u{af}",
        "sub;" => "\u{2282}",
        "subE;" => "\u{2ac5}",
        "subdot;" => "\u{2abd}",
        "sube;" => "\u{2286}",
        "subedot;" => "\u{2ac3}",
        "submult;" => "\u{2ac1}",
        "subnE;" => "\u{2acb}",
        "subne;" => "\u{228a}",
        "subplus;" => "\u{2abf}",
        "subrarr;" => "\u{2979}",
        "subset;" => "\u{2282}",
        "subseteq;" => "\u{2286}",
        "subseteqq;" => "\u{2ac5}",
        "subsetneq;" => "\u{228a}",
        "subsetneqq;" => "\u{2acb}",
        "subsim;" => "\u{2ac7}",
        "subsub;" => "\u{2ad5}",
        "subsup;" => "\u{2ad3}",
        "succ;" => "\u{227b}",
        "succapprox;" => "\u{2ab8}",
        "succcurlyeq;" => "\u{227d}",
        "succeq;" => "\u{2ab0}",
        "succnapprox;" => "\u{2aba}",
        "succneqq;" => "\u{2ab6}",
        "succnsim;" => "\u{22e9}",
        "succsim;" => "\u{227f}",
        "sum;" => "\u{2211}",
        "sung;" => "\u{266a}",
        "sup1" => "\u{b9}",
        "sup1;" => "\u{b9}",
        "sup2" => "\u{b2}",
        "sup2;" => "\u{b2}",
        "sup3" => "\u{b3}",
        "sup3;" => "\u{b3}",
        "sup;" => "\u{2283}",
        "supE;" => "\u{2ac6}",
        "supdot;" => "\u{2abe}",
        "supdsub;" => "\u{2ad8}",
        "supe;" => "\u{2287}",
        "supedot;" => "\u{2ac4}",
        "suphsol;" => "\u{27c9}",
        "suphsub;" => "\u{2ad7}",
        "suplarr;" => "\u{297b}",
        "supmult;" => "\u{2ac2}",
        "supnE;" => "\u{2acc}",
        "supne;" => "\u{228b}",
        "supplus;" => "\u{2ac0}",
        "supset;" => "\u{2283}",
        "supseteq;" => "\u{2287}",
        "supseteqq;" => "\u{2ac6}",
        "supsetneq;" => "\u{228b}",
        "supsetneqq;" => "\u{2acc}",
        "supsim;" => "\u{2ac8}",
        "supsub;" => "\u{2ad4}",
        "supsup;" => "\u{2ad6}",
        "swArr;" => "\u{21d9}",
        "swarhk;" => "\u{2926}",
        "swarr;" => "\u{2199}",
        "swarrow;" => "\u{2199}",
        "swnwar;" => "\u{292a}",
        "szlig" => "\u{df}",
        "szlig;" => "\u{df}",
        "target;" => "\u{2316}",
        "tau;" => "\u{3c4}",
        "tbrk;" => "\u{23b4}",
        "tcaron;" => "\u{165}",
        "tcedil;" => "\u{163}",
        "tcy;" => "\u{442}",
        "tdot;" => "\u{20db}",
        "telrec;" => "\u{2315}",
        "tfr;" => "\u{1d531}",
        "there4;" => "\u{2234}",
        "therefore;" => "\u{2234}",
        "theta;" => "\u{3b8}",
        "thetasym;" => "\u{3d1}",
        "thetav;" => "\u{3d1}",
        "thickapprox;" => "\u{2248}",
        "thicksim;" => "\u{223c}",
        "thinsp;" => "\u{2009}",
        "thkap;" => "\u{2248}",
        "thksim;" => "\u{223c}",
        "thorn" => "\u{fe}",
        "thorn;" => "\u{fe}",
        "tilde;" => "\u{2dc}",
        "times" => "\u{d7}",
        "times;" => "\u{d7}",
        "timesb;" => "\u{22a0}",
        "timesbar;" => "\u{2a31}",
        "timesd;" => "\u{2a30}",
        "tint;" => "\u{222d}",
        "toea;" => "\u{2928}",
        "top;" => "\u{22a4}",
        "topbot;" => "\u{2336}",
        "topcir;" => "\u{2af1}",
        "topf;" => "\u{1d565}",
        "topfork;" => "\u{2ada}",
        "tosa;" => "\u{2929}",
        "tprime;" => "\u{2034}",
        "trade;" => "\u{2122}",
        "triangle;" => "\u{25b5}",
        "triangledown;" => "\u{25bf}",
        "triangleleft;" => "\u{25c3}",
        "trianglelefteq;" => "\u{22b4}",
        "triangleq;" => "\u{225c}",
        "triangleright;" => "\u{25b9}",
        "trianglerighteq;" => "\u{22b5}",
        "tridot;" => "\u{25ec}",
        "trie;" => "\u{225c}",
        "triminus;" => "\u{2a3a}",
        "triplus;" => "\u{2a39}",
        "trisb;" => "\u{29cd}",
        "tritime;" => "\u{2a3b}",
        "trpezium;" => "\u{23e2}",
        "tscr;" => "\u{1d4c9}",
        "tscy;" => "\u{446}",
        "tshcy;" => "\u{45b}",
        "tstrok;" => "\u{167}",
        "twixt;" => "\u{226c}",
        "twoheadleftarrow;" => "\u{219e}",
        "twoheadrightarrow;" => "\u{21a0}",
        "uArr;" => "\u{21d1}",
        "uHar;" => "\u{2963}",
        "uacute" => "\u{fa}",
        "uacute;" => "\u{fa}",
        "uarr;" => "\u{2191}",
        "ubrcy;" => "\u{45e}",
        "ubreve;" => "\u{16d}",
        "ucirc" => "\u{fb}",
        "ucirc;" => "\u{fb}",
        "ucy;" => "\u{443}",
        "udarr;" => "\u{21c5}",
        "udblac;" => "\u{171}",
        "udhar;" => "\u{296e}",
        "ufisht;" => "\u{297e}",
        "ufr;" => "\u{1d532}",
        "ugrave" => "\u{f9}",
        "ugrave;" => "\u{f9}",
        "uharl;" => "\u{21bf}",
        "uharr;" => "\u{21be}",
        "uhblk;" => "\u{2580}",
        "ulcorn;" => "\u{231c}",
        "ulcorner;" => "\u{231c}",
        "ulcrop;" => "\u{230f}",
        "ultri;" => "\u{25f8}",
        "umacr;" => "\u{16b}",
        "uml" => "\u{a8}",
        "uml;" => "\u{a8}",
        "uogon;" => "\u{173}",
        "uopf;" => "\u{1d566}",
        "uparrow;" => "\u{2191}",
        "updownarrow;" => "\u{2195}",
        "upharpoonleft;" => "\u{21bf}",
        "upharpoonright;" => "\u{21be}",
        "uplus;" => "\u{228e}",
        "upsi;" => "\u{3c5}",
        "upsih;" => "\u{3d2}",
        "upsilon;" => "\u{3c5}",
        "upuparrows;" => "\u{21c8}",
        "urcorn;" => "\u{231d}",
        "urcorner;" => "\u{231d}",
        "urcrop;" => "\u{230e}",
        "uring;" => "\u{16f}",
        "urtri;" => "\u{25f9}",
        "uscr;" => "\u{1d4ca}",
        "utdot;" => "\u{22f0}",
        "utilde;" => "\u{169}",
        "utri;" => "\u{25b5}",
        "utrif;" => "\u{25b4}",
        "uuarr;" => "\u{21c8}",
        "uuml" => "\u{fc}",
        "uuml;" => "\u{fc}",
        "uwangle;" => "\u{29a7}",
        "vArr;" => "\u{21d5}",
        "vBar;" => "\u{2ae8}",
        "vBarv;" => "\u{2ae9}",
        "vDash;" => "\u{22a8}",
        "vangrt;" => "\u{299c}",
        "varepsilon;" => "\u{3f5}",
        "varkappa;" => "\u{3f0}",
        "varnothing;" => "\u{2205}",
        "varphi;" => "\u{3d5}",
        "varpi;" => "\u{3d6}",
        "varpropto;" => "\u{221d}",
        "varr;" => "\u{2195}",
        "varrho;" => "\u{3f1}",
        "varsigma;" => "\u{3c2}",
        "varsubsetneq;" => "\u{228a}\u{fe00}",
        "varsubsetneqq;" => "\u{2acb}\u{fe00}",
        "varsupsetneq;" => "\u{228b}\u{fe00}",
        "varsupsetneqq;" => "\u{2acc}\u{fe00}",
        "vartheta;" => "\u{3d1}",
        "vartriangleleft;" => "\u{22b2}",
        "vartriangleright;" => "\u{22b3}",
        "vcy;" => "\u{432}",
        "vdash;" => "\u{22a2}",
        "vee;" => "\u{2228}",
        "veebar;" => "\u{22bb}",
        "veeeq;" => "\u{225a}",
        "vellip;" => "\u{22ee}",
        "verbar;" => "\u{7c}",
        "vert;" => "\u{7c}",
        "vfr;" => "\u{1d533}",
        "vltri;" => "\u{22b2}",
        "vnsub;" => "\u{2282}\u{20d2}",
        "vnsup;" => "\u{2283}\u{20d2}",
        "vopf;" => "\u{1d567}",
        "vprop;" => "\u{221d}",
        "vrtri;" => "\u{22b3}",
        "vscr;" => "\u{1d4cb}",
        "vsubnE;" => "\u{2acb}\u{fe00}",
        "vsubne;" => "\u{228a}\u{fe00}",
        "vsupnE;" => "\u{2acc}\u{fe00}",
        "vsupne;" => "\u{228b}\u{fe00}",
        "vzigzag;" => "\u{299a}",
        "wcirc;" => "\u{175}",
        "wedbar;" => "\u{2a5f}",
        "wedge;" => "\u{2227}",
        "wedgeq;" => "\u{2259}",
        "weierp;" => "\u{2118}",
        "wfr;" => "\u{1d534}",
        "wopf;" => "\u{1d568}",
        "wp;" => "\u{2118}",
        "wr;" => "\u{2240}",
        "wreath;" => "\u{2240}",
        "wscr;" => "\u{1d4cc}",
        "xcap;" => "\u{22c2}",
        "xcirc;" => "\u{25ef}",
        "xcup;" => "\u{22c3}",
        "xdtri;" => "\u{25bd}",
        "xfr;" => "\u{1d535}",
        "xhArr;" => "\u{27fa}",
        "xharr;" => "\u{27f7}",
        "xi;" => "\u{3be}",
        "xlArr;" => "\u{27f8}",
        "xlarr;" => "\u{27f5}",
        "xmap;" => "\u{27fc}",
        "xnis;" => "\u{22fb}",
        "xodot;" => "\u{2a00}",
        "xopf;" => "\u{1d569}",
        "xoplus;" => "\u{2a01}",
        "xotime;" => "\u{2a02}",
        "xrArr;" => "\u{27f9}",
        "xrarr;" => "\u{27f6}",
        "xscr;" => "\u{1d4cd}",
        "xsqcup;" => "\u{2a06}",
        "xuplus;" => "\u{2a04}",
        "xutri;" => "\u{25b3}",
        "xvee;" => "\u{22c1}",
        "xwedge;" => "\u{22c0}",
        "yacute" => "\u{fd}",
        "yacute;" => "\u{fd}",
        "yacy;" => "\u{44f}",
        "ycirc;" => "\u{177}",
        "ycy;" => "\u{44b}",
        "yen" => "\u{a5}",
        "yen;" => "\u{a5}",
        "yfr;" => "\u{1d536}",
        "yicy;" => "\u{457}",
        "yopf;" => "\u{1d56a}",
        "yscr;" => "\u{1d4ce}",
        "yucy;" => "\u{44e}",
        "yuml" => "\u{ff}",
        "yuml;" => "\u{ff}",
        "zacute;" => "\u{17a}",
        "zcaron;" => "\u{17e}",
        "zcy;" => "\u{437}",
        "zdot;" => "\u{17c}",
        "zeetrf;" => "\u{2128}",
        "zeta;" => "\u{3b6}",
        "zfr;" => "\u{1d537}",
        "zhcy;" => "\u{436}",
        "zigrarr;" => "\u{21dd}",
        "zopf;" => "\u{1d56b}",
        "zscr;" => "\u{1d4cf}",
        "zwj;" => "\u{200d}",
        "zwnj;" => "\u{200c}",
        _ => return None,
    })
}

/// Split a CSS list only at the outer syntactic level.
pub fn split_top_level(input: &str, separator: char) -> Vec<&str> {
    let mut result = vec![];
    let mut start = 0;
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for (i, c) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        if matches!(c, '\'' | '"') {
            quote = Some(c);
            continue;
        }
        if matches!(c, '(' | '[' | '{') {
            depth += 1;
        } else if matches!(c, ')' | ']' | '}') {
            depth -= 1;
        } else if c == separator && depth == 0 {
            result.push(input[start..i].trim());
            start = i + c.len_utf8();
        }
    }
    result.push(input[start..].trim());
    result
}
pub fn matches_selector(doc: &Document, id: NodeId, selector: &str) -> bool {
    matches_selector_with_budget(doc, id, selector, &mut 100_000)
}
pub fn matches_selector_with_budget(
    doc: &Document,
    id: NodeId,
    selector: &str,
    budget: &mut usize,
) -> bool {
    if selector.len() > 4096 || doc.tag(id).is_none() || *budget == 0 {
        return false;
    }
    crate::selectors::parse_list(selector, budget).is_ok_and(|selectors| {
        selectors
            .iter()
            .any(|selector| matches_compiled_selector(doc, id, &selector.source, budget))
    })
}
pub(crate) fn matches_compiled_selector(
    doc: &Document,
    id: NodeId,
    selector: &str,
    budget: &mut usize,
) -> bool {
    doc.tag(id).is_some() && matches_complex(doc, id, selector, 0, budget)
}

fn matches_complex(doc: &Document, id: NodeId, s: &str, depth: usize, budget: &mut usize) -> bool {
    if depth > 64 {
        return false;
    }
    if *budget < s.len().max(1) {
        *budget = 0;
        return false;
    }
    *budget -= s.len().max(1);
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let mut level = 0i32;
    let mut quote = None;
    let mut escaped = false;
    let mut split = None;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        if matches!(c, '\'' | '"') {
            quote = Some(c);
            continue;
        }
        if matches!(c, '(' | '[') {
            level += 1;
        } else if matches!(c, ')' | ']') {
            level -= 1;
        } else if level == 0 && (c.is_ascii_whitespace() || matches!(c, '>' | '+' | '~')) {
            split = Some((i, c));
        }
    }
    if let Some((i, c)) = split {
        let right = s[i + c.len_utf8()..].trim();
        if right.is_empty() {
            return false;
        }
        let mut left = s[..i].trim_end();
        let mut combinator = c;
        if c.is_ascii_whitespace()
            && let Some(last) = left.chars().last()
            && matches!(last, '>' | '+' | '~')
        {
            combinator = last;
            left = left[..left.len() - 1].trim_end();
        }
        if !matches_compound(doc, id, right, depth + 1, budget) {
            return false;
        }
        match combinator {
            '>' => doc
                .nodes
                .get(id)
                .and_then(|n| n.parent)
                .is_some_and(|p| matches_complex(doc, p, left, depth + 1, budget)),
            '+' => previous_element(doc, id, budget)
                .is_some_and(|p| matches_complex(doc, p, left, depth + 1, budget)),
            '~' => {
                let mut prev = previous_element(doc, id, budget);
                let mut count = 0;
                while let Some(p) = prev {
                    if matches_complex(doc, p, left, depth + 1, budget) {
                        return true;
                    }
                    prev = previous_element(doc, p, budget);
                    count += 1;
                    if count > MAX_NODES || *budget == 0 {
                        break;
                    }
                }
                false
            }
            _ => {
                let mut parent = doc.nodes.get(id).and_then(|n| n.parent);
                let mut count = 0;
                while let Some(p) = parent {
                    if matches_complex(doc, p, left, depth + 1, budget) {
                        return true;
                    }
                    parent = doc.nodes.get(p).and_then(|n| n.parent);
                    count += 1;
                    if count > MAX_DEPTH || *budget == 0 {
                        break;
                    }
                }
                false
            }
        }
    } else {
        matches_compound(doc, id, s, depth + 1, budget)
    }
}
fn ident_end(s: &str, start: usize) -> usize {
    let mut end = start;
    for (offset, c) in s[start..].char_indices() {
        if c.is_alphanumeric() || matches!(c, '-' | '_') || !c.is_ascii() {
            end = start + offset + c.len_utf8();
        } else {
            break;
        }
    }
    end
}
fn matches_compound(doc: &Document, id: NodeId, s: &str, depth: usize, budget: &mut usize) -> bool {
    if depth > 64 || *budget == 0 {
        return false;
    }
    *budget -= 1;
    let Some(tag) = doc.tag(id) else {
        return false;
    };
    let mut i = 0;
    if s.starts_with('*') {
        i = 1;
    } else {
        let end = ident_end(s, 0);
        if end > 0 {
            if if doc.namespace(id) == Some(Namespace::Html) {
                !tag.eq_ignore_ascii_case(&s[..end])
            } else {
                tag != &s[..end]
            } {
                return false;
            }
            i = end;
        }
    }
    while i < s.len() {
        let c = s.as_bytes()[i];
        i += 1;
        match c {
            b'#' | b'.' => {
                let end = ident_end(s, i);
                if end == i {
                    return false;
                }
                let value = &s[i..end];
                let found = if c == b'#' {
                    doc.attr(id, "id") == Some(value)
                } else if let Some(classes) = doc.attr(id, "class") {
                    if classes.len() > *budget {
                        *budget = 0;
                        return false;
                    }
                    *budget -= classes.len();
                    classes.split_ascii_whitespace().any(|x| x == value)
                } else {
                    false
                };
                if !found {
                    return false;
                }
                i = end;
            }
            b'[' => {
                let Some(end) = find_balanced_end(s, i, b'[', b']') else {
                    return false;
                };
                if !matches_attr(doc, id, &s[i..end], budget) {
                    return false;
                }
                i = end + 1;
            }
            b':' => {
                if s.as_bytes().get(i) == Some(&b':') {
                    return false;
                }
                let end = ident_end(s, i);
                if end == i {
                    return false;
                }
                let name = &s[i..end];
                i = end;
                let mut argument = "";
                if s.as_bytes().get(i) == Some(&b'(') {
                    let Some(end) = find_balanced_end(s, i + 1, b'(', b')') else {
                        return false;
                    };
                    argument = s[i + 1..end].trim();
                    i = end + 1;
                }
                let sibling_children = doc
                    .nodes
                    .get(id)
                    .and_then(|n| n.parent)
                    .and_then(|p| doc.nodes.get(p))
                    .map(|n| n.children.as_slice())
                    .unwrap_or(&[]);
                let needs_siblings = matches!(
                    name,
                    "first-child"
                        | "last-child"
                        | "only-child"
                        | "first-of-type"
                        | "last-of-type"
                        | "only-of-type"
                        | "nth-child"
                        | "nth-last-child"
                        | "nth-of-type"
                        | "nth-last-of-type"
                        | "empty"
                );
                if needs_siblings {
                    if sibling_children.len() > *budget {
                        *budget = 0;
                        return false;
                    }
                    *budget -= sibling_children.len();
                }
                let siblings = || -> Vec<NodeId> {
                    sibling_children
                        .iter()
                        .copied()
                        .filter(|n| doc.tag(*n).is_some())
                        .collect()
                };
                let ok = match name {
                    "root" => tag == "html",
                    "scope" => tag == "html",
                    "not" => !split_top_level(argument, ',')
                        .into_iter()
                        .any(|s| matches_complex(doc, id, s, depth + 1, budget)),
                    "is" | "where" => split_top_level(argument, ',')
                        .into_iter()
                        .any(|s| matches_complex(doc, id, s, depth + 1, budget)),
                    "first-child" => siblings().first() == Some(&id),
                    "last-child" => siblings().last() == Some(&id),
                    "only-child" => siblings() == vec![id],
                    "first-of-type" => {
                        siblings().into_iter().find(|n| {
                            doc.tag(*n) == Some(tag) && doc.namespace(*n) == doc.namespace(id)
                        }) == Some(id)
                    }
                    "last-of-type" => {
                        siblings().into_iter().rev().find(|n| {
                            doc.tag(*n) == Some(tag) && doc.namespace(*n) == doc.namespace(id)
                        }) == Some(id)
                    }
                    "only-of-type" => {
                        siblings()
                            .into_iter()
                            .filter(|n| {
                                doc.tag(*n) == Some(tag) && doc.namespace(*n) == doc.namespace(id)
                            })
                            .count()
                            == 1
                    }
                    "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type" => {
                        let mut all = siblings();
                        if name.contains("of-type") {
                            all.retain(|n| {
                                doc.tag(*n) == Some(tag) && doc.namespace(*n) == doc.namespace(id)
                            });
                        }
                        if name.contains("last") {
                            all.reverse();
                        }
                        all.iter()
                            .position(|n| *n == id)
                            .is_some_and(|n| nth_matches(argument, n + 1))
                    }
                    "empty" => doc.nodes[id]
                        .children
                        .iter()
                        .all(|n| match &doc.nodes[*n].kind {
                            NodeKind::Text(t) => t.is_empty(),
                            NodeKind::Element(_) => false,
                            _ => true,
                        }),
                    "checked" => {
                        doc.namespace(id) == Some(Namespace::Html)
                            && (tag == "input" && doc.attr(id, "checked").is_some()
                                || tag == "option" && doc.attr(id, "selected").is_some())
                    }
                    "disabled" => {
                        doc.namespace(id) == Some(Namespace::Html)
                            && matches!(
                                tag,
                                "input"
                                    | "select"
                                    | "textarea"
                                    | "button"
                                    | "option"
                                    | "optgroup"
                                    | "fieldset"
                            )
                            && doc.attr(id, "disabled").is_some()
                    }
                    "enabled" => {
                        doc.namespace(id) == Some(Namespace::Html)
                            && matches!(
                                tag,
                                "input"
                                    | "select"
                                    | "textarea"
                                    | "button"
                                    | "option"
                                    | "optgroup"
                                    | "fieldset"
                            )
                            && doc.attr(id, "disabled").is_none()
                    }
                    "required" => {
                        doc.namespace(id) == Some(Namespace::Html)
                            && matches!(tag, "input" | "select" | "textarea")
                            && doc.attr(id, "required").is_some()
                    }
                    "optional" => {
                        doc.namespace(id) == Some(Namespace::Html)
                            && matches!(tag, "input" | "select" | "textarea")
                            && doc.attr(id, "required").is_none()
                    }
                    "link" | "any-link" => {
                        (doc.namespace(id) == Some(Namespace::Html)
                            && matches!(tag, "a" | "area" | "link")
                            || doc.namespace(id) == Some(Namespace::Svg) && tag == "a")
                            && doc.attr(id, "href").is_some()
                    }
                    "lang" => {
                        let lang = if argument.starts_with(['\'', '"']) {
                            &argument[1..argument.len() - 1]
                        } else {
                            argument
                        };
                        let mut n = Some(id);
                        let mut found = false;
                        let mut count = 0;
                        while let Some(p) = n {
                            let xml_language = match doc.nodes.get(p).map(|node| &node.kind) {
                                Some(NodeKind::Element(el))
                                    if el.attr_namespaces.get("xml:lang")
                                        == Some(&AttributeNamespace::Xml) =>
                                {
                                    el.attrs.get("xml:lang").map(String::as_str)
                                }
                                _ => None,
                            };
                            if let Some(v) = xml_language.or_else(|| doc.attr(p, "lang")) {
                                if v.len() > *budget {
                                    *budget = 0;
                                    return false;
                                }
                                *budget -= v.len();
                                found = v.eq_ignore_ascii_case(lang)
                                    || v.to_ascii_lowercase()
                                        .starts_with(&format!("{}-", lang.to_ascii_lowercase()));
                                break;
                            }
                            n = doc.nodes.get(p).and_then(|n| n.parent);
                            count += 1;
                            if count > MAX_DEPTH {
                                break;
                            }
                        }
                        found
                    }
                    _ => false,
                };
                if !ok || *budget == 0 {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}
fn find_balanced_end(s: &str, start: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 1;
    let mut quote = None;
    let mut escape = false;
    for (i, b) in s.bytes().enumerate().skip(start) {
        if escape {
            escape = false;
            continue;
        }
        if b == b'\\' {
            escape = true;
            continue;
        }
        if let Some(q) = quote {
            if b == q {
                quote = None;
            }
            continue;
        }
        if matches!(b, b'\'' | b'"') {
            quote = Some(b);
            continue;
        }
        if b == open {
            depth += 1;
        } else if b == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}
fn matches_attr(doc: &Document, id: NodeId, s: &str, budget: &mut usize) -> bool {
    let s = s.trim();
    let Some(eq) = s.find('=') else {
        return doc.attr(id, s).is_some();
    };
    let (name, op) = if eq > 0 && matches!(s.as_bytes()[eq - 1], b'~' | b'|' | b'^' | b'$' | b'*') {
        (s[..eq - 1].trim(), s.as_bytes()[eq - 1])
    } else {
        (s[..eq].trim(), b'=')
    };
    let Some(actual) = doc.attr(id, name) else {
        return false;
    };
    if actual.len() > *budget {
        *budget = 0;
        return false;
    }
    *budget -= actual.len();
    let raw = s[eq + 1..].trim();
    let insensitive = raw.ends_with(" i") || raw.ends_with(" I");
    let raw = if insensitive || raw.ends_with(" s") {
        raw[..raw.len() - 2].trim()
    } else {
        raw
    };
    let expected = if raw.starts_with(['\'', '"']) {
        &raw[1..raw.len() - 1]
    } else {
        raw
    };
    let (actual, expected) = if insensitive {
        (actual.to_ascii_lowercase(), expected.to_ascii_lowercase())
    } else {
        (actual.to_owned(), expected.to_owned())
    };
    match op {
        b'=' => actual == expected,
        b'~' => !expected.is_empty() && actual.split_ascii_whitespace().any(|x| x == expected),
        b'|' => actual == expected || actual.starts_with(&format!("{expected}-")),
        b'^' => !expected.is_empty() && actual.starts_with(&expected),
        b'$' => !expected.is_empty() && actual.ends_with(&expected),
        b'*' => !expected.is_empty() && actual.contains(&expected),
        _ => false,
    }
}
fn previous_element(doc: &Document, id: NodeId, budget: &mut usize) -> Option<NodeId> {
    let parent = doc.nodes.get(id)?.parent?;
    let mut previous = None;
    for child in &doc.nodes.get(parent)?.children {
        if *budget == 0 {
            return None;
        }
        *budget -= 1;
        if *child == id {
            return previous;
        }
        if doc.tag(*child).is_some() {
            previous = Some(*child);
        }
    }
    None
}
fn nth_matches(s: &str, index: usize) -> bool {
    let s = s.to_ascii_lowercase().replace(' ', "");
    if s == "odd" {
        return index % 2 == 1;
    }
    if s == "even" {
        return index.is_multiple_of(2);
    }
    if let Some(n) = s.find('n') {
        let a = match &s[..n] {
            "" | "+" => 1,
            "-" => -1,
            s => match s.parse::<i64>() {
                Ok(n) => n,
                Err(_) => return false,
            },
        };
        let b = if n + 1 == s.len() {
            0
        } else {
            match s[n + 1..].parse::<i64>() {
                Ok(n) => n,
                Err(_) => return false,
            }
        };
        // Parsed coefficients are i64; wider arithmetic keeps their full
        // mathematical range without overflow in subtraction or division.
        let diff = index as i128 - i128::from(b);
        let a = i128::from(a);
        a == 0 && diff == 0 || a != 0 && diff % a == 0 && diff / a >= 0
    } else {
        s.parse::<usize>().ok() == Some(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details_summary_cache_and_visibility_follow_mutations_without_inerting_content() {
        let mut d = Document::parse(
            "<details id=d><p id=before>x</p><summary id=a>A</summary><summary id=b>B</summary><input id=input name=v value=ok><script id=script>value</script></details>",
        );
        let details = d.query_selector("#d").unwrap();
        let a = d.query_selector("#a").unwrap();
        let b = d.query_selector("#b").unwrap();
        let input = d.query_selector("#input").unwrap();
        assert_eq!(d.first_summary(details), Some(a));
        assert!(!d.disclosure_hidden(a));
        assert!(d.disclosure_hidden(b));
        assert!(d.is_active_node(input));
        assert!(!d.disabled_control(input));
        assert!(!d.can_edit_control(input));
        assert!(!d.can_focus_control(input));
        d.append_child(details, a);
        assert_eq!(d.first_summary(details), Some(b));
        assert!(d.disclosure_hidden(a));
        d.set_attr(details, "open", "false");
        assert!(!d.disclosure_hidden(input));
        assert!(d.can_edit_control(input));
        d.clear_children(details);
        assert_eq!(d.first_summary(details), None);
    }

    #[test]
    fn details_name_groups_follow_attributes_detached_roots_and_bulk_moves() {
        let mut d = Document::parse(
            "<details id=a name=g open></details><details id=b name=g open></details><details id=c name=G open></details>",
        );
        let a = d.query_selector("#a").unwrap();
        let b = d.query_selector("#b").unwrap();
        let c = d.query_selector("#c").unwrap();
        assert!(d.attr(a, "open").is_some());
        assert!(d.attr(b, "open").is_none());
        assert!(d.attr(c, "open").is_some());
        d.set_attr(b, "open", "");
        assert!(d.attr(a, "open").is_none());
        assert!(d.attr(b, "open").is_some());
        d.set_attr(c, "name", "g");
        assert!(d.attr(c, "open").is_none());
        let fragment = d.create_document_fragment();
        d.append_child(fragment, b);
        d.set_attr(a, "open", "");
        assert!(
            d.attr(b, "open").is_some(),
            "separate trees have separate groups"
        );
        let body = d.query_selector("body").unwrap();
        d.append_child(body, fragment);
        assert!(
            d.attr(b, "open").is_none(),
            "existing destination member wins insertion"
        );
        assert_eq!(d.details_groups.len(), 1);
        d.clear_children(body);
        d.set_attr(b, "open", "");
        assert!(d.attr(a, "open").is_some());
        assert!(d.attr(b, "open").is_some());
    }

    #[test]
    fn details_toggle_queue_coalesces_reorders_and_does_not_grow_with_repeated_toggles() {
        let mut d = Document::parse("<details id=a></details><details id=b></details>");
        let a = d.query_selector("#a").unwrap();
        let b = d.query_selector("#b").unwrap();
        d.set_attr(a, "open", "");
        d.set_attr(b, "open", "");
        d.remove_attr(a, "open");
        let task = d.peek_details_toggle().unwrap();
        let event = d.begin_details_toggle(task.id).unwrap().event;
        assert!(d.finish_details_toggle(task.id));
        assert_eq!(event.node, b);
        let task = d.peek_details_toggle().unwrap();
        let event = d.begin_details_toggle(task.id).unwrap().event;
        assert!(d.finish_details_toggle(task.id));
        assert_eq!(
            (event.node, event.old_open, event.new_open),
            (a, false, false)
        );
        d.set_attr(b, "open", "different value");
        assert!(d.peek_details_toggle().is_none());
        for _ in 0..20_000 {
            d.set_attr(a, "open", "");
            d.remove_attr(a, "open");
        }
        assert_eq!(d.details_trackers.len(), 1);
        assert_eq!(d.details_toggles.len(), 1);
    }

    #[test]
    fn details_task_identity_preserves_tracker_through_reentrant_mutations_and_finish() {
        let mut d = Document::parse("<details id=d></details>");
        let node = d.query_selector("#d").unwrap();
        d.set_attr(node, "open", "");
        let first = d.peek_details_toggle().unwrap();
        assert!(d.begin_details_toggle(first.id + 1).is_none());
        assert_eq!(d.peek_details_toggle(), Some(first));
        assert_eq!(d.begin_details_toggle(first.id), Some(first));
        assert!(d.details_toggle_running());
        assert_eq!(d.details_trackers[&node].task, first.id);
        assert!(!d.has_pending_details_toggles());
        d.remove_attr(node, "open");
        let replacement = d.peek_details_toggle().unwrap();
        assert_ne!(replacement.id, first.id);
        assert_eq!(
            (replacement.event.old_open, replacement.event.new_open),
            (false, false)
        );
        assert_eq!(d.details_trackers[&node].task, replacement.id);
        assert!(d.begin_details_toggle(replacement.id).is_none());
        assert!(!d.finish_details_toggle(replacement.id));
        assert_eq!(d.details_trackers[&node].task, replacement.id);
        assert!(d.finish_details_toggle(first.id));
        assert!(!d.details_toggle_running());
        assert!(d.details_trackers.is_empty());
        assert_eq!(d.peek_details_toggle(), Some(replacement));
        assert!(!d.finish_details_toggle(first.id));
        d.begin_details_toggle(replacement.id).unwrap();
        assert!(
            d.details_trackers.is_empty(),
            "starting an untracked task does not recreate a tracker"
        );
        assert!(d.finish_details_toggle(replacement.id));
    }

    #[test]
    fn details_untracked_task_preserves_and_coalesces_with_a_newer_tracker() {
        let mut d = Document::parse("<details id=d></details>");
        let node = d.query_selector("#d").unwrap();
        d.set_attr(node, "open", "");
        let first = d.peek_details_toggle().unwrap();
        d.begin_details_toggle(first.id).unwrap();
        d.remove_attr(node, "open");
        d.set_attr(node, "open", "");
        let second = d.peek_details_toggle().unwrap();
        assert_eq!(
            (second.event.old_open, second.event.new_open),
            (false, true)
        );
        assert!(d.finish_details_toggle(first.id));
        d.remove_attr(node, "open");
        assert_eq!(d.details_toggles.len(), 2);
        let third = d.details_trackers[&node].task;
        assert!(d.details_toggles[&third].old_open);
        d.begin_details_toggle(second.id).unwrap();
        assert_eq!(d.details_trackers[&node].task, third);
        d.set_attr(node, "open", "");
        let fourth = d.peek_details_toggle().unwrap();
        assert!(!d.details_toggles.contains_key(&third));
        assert_ne!(fourth.id, third);
        assert_eq!((fourth.event.old_open, fourth.event.new_open), (true, true));
        assert!(d.finish_details_toggle(second.id));
        assert!(d.details_trackers.is_empty());
        assert_eq!(d.peek_details_toggle(), Some(fourth));
        d.begin_details_toggle(fourth.id).unwrap();
        assert!(d.finish_details_toggle(fourth.id));
        assert!(!d.has_pending_details_toggles());
    }

    #[test]
    fn details_task_queue_retains_at_most_two_live_records_per_element() {
        let mut d = Document::parse(&"<details></details>".repeat(8));
        let nodes = d.query_selector_all("details");
        let mut random = 0x544f4747u64;
        for _ in 0..20_000 {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            match random % 4 {
                0 | 1 => {
                    let node = nodes[(random as usize >> 8) % nodes.len()];
                    if d.attr(node, "open").is_some() {
                        d.remove_attr(node, "open");
                    } else {
                        d.set_attr(node, "open", "");
                    }
                }
                2 => {
                    if let Some(task) = d.peek_details_toggle() {
                        d.begin_details_toggle(task.id);
                    }
                }
                _ => {
                    if let Some(task) = d.details_active {
                        assert!(d.finish_details_toggle(task.id));
                    }
                }
            }
            let mut counts = BTreeMap::<NodeId, usize>::new();
            for task in d.details_toggles.values() {
                *counts.entry(task.node).or_default() += 1;
            }
            if let Some(task) = d.details_active {
                *counts.entry(task.event.node).or_default() += 1;
            }
            assert!(counts.values().all(|&count| count <= 2));
            assert!(d.details_trackers.len() <= nodes.len());
            for tracker in d.details_trackers.values() {
                assert!(
                    d.details_toggles.contains_key(&tracker.task)
                        || d.details_active.is_some_and(|task| task.id == tracker.task)
                );
            }
        }
    }
    #[test]
    fn details_first_summary_lookup_is_shared_across_broad_sibling_sets() {
        let source = format!(
            "<details>{}{}",
            "<i></i>".repeat(10_000),
            "<summary></summary>".repeat(10_000)
        );
        let mut d = Document::parse(&source);
        let details = d.query_selector("details").unwrap();
        let summaries = d.query_selector_all("summary");
        for &summary in &summaries {
            assert_eq!(
                d.summary_details(summary),
                (summary == summaries[0]).then_some(details)
            );
        }
        assert_eq!(d.details_summaries.borrow().len(), 1);
        d.remove_child(details, summaries[0]);
        assert_eq!(d.first_summary(details), Some(summaries[1]));
    }
    #[test]
    fn child_text_collection_preflights_bytes_and_visits_without_descending() {
        let mut document = Document::parse("");
        let style = document.create_element("style");
        let first = document.create_text_node("aé");
        let comment = document.create_comment("comment");
        let descendant = document.create_element("span");
        document.set_text_content(descendant, "descendant");
        let last = document.create_text_node("𝄞z");
        for child in [first, comment, descendant, last] {
            document.append_child(style, child);
        }
        let mut bytes = 8;
        let mut visits = 8;
        assert_eq!(
            document.child_text_content_bounded(style, &mut bytes, &mut visits),
            Some("aé𝄞z".into())
        );
        assert_eq!((bytes, visits), (0, 0));
        assert_eq!(document.text_content(style), "aédescendant𝄞z");
        let mut bytes = 7;
        let mut visits = 8;
        assert!(
            document
                .child_text_content_bounded(style, &mut bytes, &mut visits)
                .is_none()
        );
        assert_eq!((bytes, visits), (7, 0));
        let mut bytes = 8;
        let mut visits = 7;
        assert!(
            document
                .child_text_content_bounded(style, &mut bytes, &mut visits)
                .is_none()
        );
        assert_eq!((bytes, visits), (8, 0));
    }
    #[test]
    fn stylesheet_collection_uses_exact_type_and_shared_source_limits() {
        let mut document = Document::parse(
            "<style type='text/css; charset=utf-8'>html parameters</style><style type=' text/css'>html whitespace</style><style type='TEXT/CSS'>html</style><svg><style type='text/css; charset=utf-8'>svg parameters</style><style type=' '>svg whitespace</style><style type=''>svg</style></svg>",
        );
        assert_eq!(document.stylesheets(), ["html", "svg"]);
        document = Document::parse(&"<style>x</style>".repeat(MAX_INLINE_STYLES + 1));
        assert_eq!(document.stylesheets().len(), MAX_INLINE_STYLES);
        let mut document =
            Document::parse("<style id=a></style><style id=b></style><style id=c>overflow</style>");
        let text = "x".repeat(MAX_TEXT / 2);
        for id in ["#a", "#b"] {
            document.set_text_content(document.query_selector(id).unwrap(), &text);
        }
        let styles = document.stylesheets();
        assert_eq!(styles.len(), 2);
        assert_eq!(styles.iter().map(String::len).sum::<usize>(), MAX_TEXT);
    }
    #[test]
    fn nested_svg_styles_do_not_multiply_descendant_text() {
        let document = Document::parse(&format!(
            "<svg>{}{}{}",
            "<style>".repeat(64),
            "x".repeat(4096),
            "</style>".repeat(64)
        ));
        let styles = document.stylesheets();
        assert_eq!(styles.len(), 64);
        assert_eq!(styles.iter().map(String::len).sum::<usize>(), 4096);
        assert!(styles[..63].iter().all(String::is_empty));
    }
    #[test]
    fn block_starts_and_end_tag_scopes_keep_paragraph_and_button_boundaries() {
        for tag in [
            "center",
            "details",
            "dialog",
            "dir",
            "figcaption",
            "figure",
            "listing",
            "menu",
            "search",
            "summary",
        ] {
            let d = Document::parse(&format!("<!doctype html><p>A<{tag}>B<p>C"));
            assert_eq!(
                d.outer_html(d.query_selector("body").unwrap()),
                format!("<body><p>A</p><{tag}>B<p>C</p></{tag}></body>")
            );
        }
        let d = Document::parse("<address><button></address>A<p><button><div>B");
        assert_eq!(
            d.outer_html(d.query_selector("body").unwrap()),
            "<body><address><button></button></address>A<p><button><div>B</div></button></p></body>"
        );
    }
    #[test]
    fn ruby_implied_end_tags_respect_rtc_and_scope_boundaries() {
        for (source, expected) in [
            (
                "<ruby>A<rb>B<rt>C<rp>D<rtc>E<rt>F<rt>G<rb>H",
                "<ruby>A<rb>B</rb><rt>C</rt><rp>D</rp><rtc>E<rt>F</rt><rt>G</rt></rtc><rb>H</rb></ruby>",
            ),
            (
                "<ruby><div><p>A<rp>B",
                "<ruby><div><p>A</p><rp>B</rp></div></ruby>",
            ),
            (
                "<ruby><table><td><p>A<rp>B",
                "<ruby><table><tbody><tr><td><p>A<rp>B</rp></p></td></tr></tbody></table></ruby>",
            ),
        ] {
            let d = Document::parse(source);
            assert_eq!(d.outer_html(d.query_selector("ruby").unwrap()), expected);
        }
    }
    #[test]
    fn list_item_scope_and_customizable_select_recovery_follow_current_rules() {
        for (source, expected) in [
            (
                "<ul><li>A<ul></li><li>B</ul>C",
                "<body><ul><li>A<ul><li>B</li></ul>C</li></ul></body>",
            ),
            (
                "<p>A<dd>B<dt>C",
                "<body><p>A</p><dd>B</dd><dt>C</dt></body>",
            ),
            (
                "<select><option>A<optgroup><option>B<hr><input>C",
                "<body><select><option>A</option><optgroup><option>B</option></optgroup><hr></select><input>C</body>",
            ),
            (
                "<select><button><select>D",
                "<body><select><button></button></select>D</body>",
            ),
            (
                "<option><span>A<option>B",
                "<body><option><span>A<option>B</option></span></option></body>",
            ),
        ] {
            let d = Document::parse(source);
            assert_eq!(
                d.outer_html(d.query_selector("body").unwrap()),
                expected,
                "{source}"
            );
        }
    }
    #[test]
    fn decoded_lf_plaintext_and_foreign_null_frameset_rules_are_distinct() {
        for tag in ["pre", "listing", "textarea"] {
            let d = Document::parse(&format!("<{tag}>&#10;&#x0a;A</{tag}>"));
            assert_eq!(d.text_content(d.query_selector(tag).unwrap()), "\nA");
        }
        let d = Document::parse("<pre><!--comment-->\nA</pre><p><b><plaintext>B");
        assert_eq!(d.text_content(d.query_selector("pre").unwrap()), "\nA");
        assert_eq!(
            d.outer_html(d.query_selector("plaintext").unwrap()),
            "<plaintext><b>B</b></plaintext>"
        );
        for source in [
            "<svg>\0 </svg><frameset>",
            "<svg><![CDATA[\0 ]]></svg><frameset>",
        ] {
            let d = Document::parse(source);
            assert!(d.query_selector("frameset").is_some());
            assert!(d.query_selector("body").is_none());
        }
        for source in ["<svg>�</svg><frameset>", "<svg>&#0;</svg><frameset>"] {
            assert!(Document::parse(source).query_selector("frameset").is_none());
        }
        let d = Document::parse("<frameset></frameset> A\tB\nC");
        assert_eq!(d.text_content(d.query_selector("html").unwrap()), " \t\n");
    }
    #[test]
    fn long_attribute_names_preserve_distinct_identity_and_owner_urls_are_inherited() {
        let first = format!("data-{}", "é".repeat(800));
        let second = format!("{first}x");
        let mut d = Document::parse(&format!("<div {first}=A {second}=B></div>"));
        let id = d.query_selector("div").unwrap();
        assert_eq!(d.attr(id, &first), Some("A"));
        assert_eq!(d.attr(id, &second), Some("B"));
        assert_eq!(d.url().as_str(), "about:blank");
        d.set_url(url::Url::parse("https://example.test/a/b?q=1#f").unwrap());
        assert_eq!(d.parse_fragment(id, "<p>x").unwrap().url(), d.url());
        assert_bounded_forest(&d);
        assert!(Document::from_snapshot(d.nodes.clone(), d.root, false, d.mode).is_ok());
    }
    #[test]
    fn selectedcontent_clones_real_option_trees_at_insertion_and_completion() {
        let d = Document::parse(
            "<select><button><selectedcontent></button><option>A<option selected><i>B</i><svg><lineargradient xlink:href='#paint'/></svg><template><b>C</b></template><!--D-->",
        );
        let selected = d.query_selector("selectedcontent").unwrap();
        let option = d.query_selector("option[selected]").unwrap();
        assert_eq!(d.text_content(selected), "B");
        assert_ne!(d.nodes[selected].children, d.nodes[option].children);
        let clone = d.query_selector_from(selected, "linearGradient").unwrap();
        assert_eq!(d.namespace(clone), Some(Namespace::Svg));
        assert!(
            matches!(&d.nodes[clone].kind, NodeKind::Element(el) if el.attr_namespaces.get("xlink:href") == Some(&AttributeNamespace::XLink))
        );
        let template = d.query_selector_from(selected, "template").unwrap();
        assert_eq!(d.text_content(d.template_contents(template).unwrap()), "C");
        assert_bounded_forest(&d);
        assert!(Document::from_snapshot(d.nodes.clone(), d.root, false, d.mode).is_ok());
        for (source, expected) in [
            (
                "<select><option selected>A<option selected>B</option><selectedcontent>",
                "B",
            ),
            (
                "<select><selectedcontent></selectedcontent><optgroup disabled><option>A</optgroup><option>B",
                "B",
            ),
            (
                "<select size=2><selectedcontent></selectedcontent><option>A",
                "",
            ),
            (
                "<select multiple><selectedcontent></selectedcontent><option selected>A",
                "",
            ),
            ("<select><option><selectedcontent></selectedcontent>A", ""),
            (
                "<template><select><selectedcontent></selectedcontent><option>A",
                "",
            ),
        ] {
            let d = Document::parse(source);
            let id = d.nodes.iter().position(|node| matches!(&node.kind, NodeKind::Element(el) if el.tag == "selectedcontent")).unwrap();
            assert_eq!(d.text_content(id), expected, "{source}");
        }
    }
    #[test]
    fn selectedcontent_clone_preflight_and_repeated_selection_share_parser_limits() {
        let mut builder = TreeBuilder::new(false);
        builder.doc = Document::parse(
            "<select><selectedcontent></selectedcontent><option>A<b>B</b></option></select>",
        );
        let target = builder.doc.query_selector("selectedcontent").unwrap();
        let option = builder.doc.query_selector("option").unwrap();
        builder.doc.set_text_content(target, "unchanged");
        let nodes = builder.doc.nodes.len();
        builder.work = 1;
        builder.clone_option_contents(option, target);
        assert_eq!(builder.doc.text_content(target), "unchanged");
        assert_eq!(builder.doc.nodes.len(), nodes);
        builder.work = 50_000;
        builder.doc.retained_bytes = MAX_DOM_BYTES;
        builder.clone_option_contents(option, target);
        assert_eq!(builder.doc.text_content(target), "unchanged");
        assert_eq!(builder.doc.nodes.len(), nodes);
        let mut builder = TreeBuilder::new(false);
        builder.work = 5000;
        let d = parse_with_builder(
            &format!(
                "<select><selectedcontent></selectedcontent>{}",
                "<option selected><b>A</b>".repeat(20_000)
            ),
            builder,
        );
        assert!(d.nodes.len() < 5000);
        assert_bounded_forest(&d);
        assert!(Document::from_snapshot(d.nodes.clone(), d.root, false, d.mode).is_ok());
    }
    fn fragment(namespace: Namespace, tag: &str, source: &str, scripting: bool) -> Document {
        let mut owner = Document::parse_with_scripting("", scripting);
        let context = owner.create_element_ns(namespace, tag);
        owner.parse_fragment(context, source).unwrap()
    }
    #[test]
    fn fragment_table_modes_use_context_without_inserting_it_on_the_stack() {
        for (context, source, expected) in [
            (
                "table",
                "<td>A<td>B",
                "<tbody><tr><td>A</td><td>B</td></tr></tbody>",
            ),
            ("tbody", "<td>A<td>B", "<tr><td>A</td><td>B</td></tr>"),
            ("tr", "<td>A<td>B", "<td>A</td><td>B</td>"),
            ("td", "one</td><td>two", "onetwo"),
            ("colgroup", "x<col><colgroup><col>", "<col><col>"),
            (
                "frameset",
                "</frameset><frame><div>ignored</div>",
                "<frame>",
            ),
            (
                "select",
                "<input><select><keygen><option>A<option>B",
                "<keygen><option>A</option><option>B</option>",
            ),
            ("div", "<span><frameset>", "<span></span>"),
            (
                "html",
                "<frameset><span>",
                "<head></head><frameset></frameset>",
            ),
            (
                "html",
                "<title>T</title><p>B</html><!--tail-->",
                "<head><title>T</title></head><body><p>B</p></body><!--tail-->",
            ),
        ] {
            let d = fragment(Namespace::Html, context, source, false);
            assert_eq!(d.outer_html(d.root), expected, "{context}");
            assert_bounded_forest(&d);
        }
    }
    #[test]
    fn fragment_tokenizer_has_no_appropriate_context_end_tag_and_keeps_initial_lf() {
        for context in [
            "textarea",
            "title",
            "style",
            "xmp",
            "iframe",
            "noembed",
            "noframes",
            "script",
            "plaintext",
            "noscript",
        ] {
            for scripting in [false, true] {
                let source = format!("\n&amp;\0</{context}></\0fragment><b>text</b>");
                let d = fragment(Namespace::Html, context, &source, scripting);
                if context == "noscript" && !scripting {
                    assert!(d.query_selector("b").is_some());
                } else {
                    let expected = source.replace('\0', "\u{fffd}");
                    let expected = if matches!(context, "textarea" | "title") {
                        expected.replace("&amp;", "&")
                    } else {
                        expected
                    };
                    assert_eq!(d.text_content(d.root), expected, "{context}/{scripting}");
                    assert_eq!(d.nodes[d.root].children.len(), 1);
                    assert!(matches!(
                        d.nodes[d.nodes[d.root].children[0]].kind,
                        NodeKind::Text(_)
                    ));
                }
                assert_bounded_forest(&d);
            }
        }
    }
    #[test]
    fn foreign_fragment_adjusted_current_node_controls_cdata_and_breakout() {
        let d = fragment(
            Namespace::Svg,
            "svg",
            "<![CDATA[<&\0]]><lineargradient xlink:href='#x' viewbox='0 0 1 1'/><p>HTML</p><circle/>",
            false,
        );
        assert!(d.text_content(d.root).starts_with("<&\u{fffd}"));
        let gradient = d.query_selector("linearGradient").unwrap();
        assert_eq!(d.namespace(gradient), Some(Namespace::Svg));
        assert_eq!(d.attr(gradient, "viewBox"), Some("0 0 1 1"));
        let NodeKind::Element(element) = &d.nodes[gradient].kind else {
            unreachable!()
        };
        assert_eq!(
            element.attr_namespaces.get("xlink:href"),
            Some(&AttributeNamespace::XLink)
        );
        assert_eq!(
            d.namespace(d.query_selector("p").unwrap()),
            Some(Namespace::Html)
        );
        assert_eq!(
            d.namespace(d.query_selector("circle").unwrap()),
            Some(Namespace::Svg)
        );
        for context in ["foreignObject", "desc", "title"] {
            let d = fragment(
                Namespace::Svg,
                context,
                "<b>HTML</b><svg><circle/></svg>",
                false,
            );
            assert_eq!(
                d.namespace(d.query_selector("b").unwrap()),
                Some(Namespace::Html)
            );
            assert_eq!(
                d.namespace(d.query_selector("circle").unwrap()),
                Some(Namespace::Svg)
            );
            assert_bounded_forest(&d);
        }
        assert_bounded_forest(&d);
    }
    #[test]
    fn math_fragment_integration_uses_context_attributes_and_namespace() {
        let d = fragment(
            Namespace::MathMl,
            "mi",
            "<mglyph/><b>HTML</b><malignmark/>",
            false,
        );
        for (selector, namespace) in [
            ("mglyph", Namespace::MathMl),
            ("b", Namespace::Html),
            ("malignmark", Namespace::MathMl),
        ] {
            assert_eq!(
                d.namespace(d.query_selector(selector).unwrap()),
                Some(namespace)
            );
        }
        let mut owner = Document::parse("");
        let context = owner.create_element_ns(Namespace::MathMl, "annotation-xml");
        owner.set_attr(context, "encoding", "TEXT/HTML");
        let d = owner.parse_fragment(context, "<div><svg/></div>").unwrap();
        assert_eq!(
            d.namespace(d.query_selector("div").unwrap()),
            Some(Namespace::Html)
        );
        assert_eq!(
            d.namespace(d.query_selector("svg").unwrap()),
            Some(Namespace::Svg)
        );
        let d = fragment(
            Namespace::MathMl,
            "annotation-xml",
            "<spanish/><svg/>",
            false,
        );
        assert_eq!(
            d.namespace(d.query_selector("spanish").unwrap()),
            Some(Namespace::MathMl)
        );
        assert_eq!(
            d.namespace(d.query_selector("svg").unwrap()),
            Some(Namespace::Svg)
        );
    }
    #[test]
    fn fragment_form_pointer_uses_inclusive_context_ancestors_without_mutating_them() {
        let owner = Document::parse("<form id=outer><div id=context></div></form>");
        let context = owner.query_selector("#context").unwrap();
        let before = owner.outer_html(owner.root);
        let d = owner
            .parse_fragment(
                context,
                "<form id=ignored><input></form><form id=allowed></form>",
            )
            .unwrap();
        assert!(d.query_selector("#ignored").is_none());
        assert!(d.query_selector("#allowed").is_some());
        assert!(d.query_selector("input").is_some());
        assert_eq!(owner.outer_html(owner.root), before);
        let form = owner.query_selector("#outer").unwrap();
        let d = owner
            .parse_fragment(form, "<form id=ignored><input>")
            .unwrap();
        assert!(d.query_selector("#ignored").is_none());
        assert_bounded_forest(&d);
    }
    #[test]
    fn fragment_rejects_invalid_contexts_and_bounds_malformed_reparenting() {
        let mut owner = Document::parse("");
        assert!(owner.parse_fragment(owner.root, "x").is_err());
        assert!(owner.parse_fragment(usize::MAX, "x").is_err());
        let unhosted = owner.create_document_fragment();
        assert!(owner.parse_fragment(unhosted, "x").is_err());
        let context = owner.create_element("div");
        owner.nodes[context].parent = Some(context);
        assert!(owner.parse_fragment(context, "x").is_err());
        for namespace in [Namespace::Html, Namespace::Svg, Namespace::MathMl] {
            let source = format!(
                "{}{}",
                "<a><b><div><table><svg><foreignObject>".repeat(800),
                "</a></table></svg><p>tail".repeat(800)
            );
            let d = fragment(namespace, "div", &source, false);
            assert_bounded_forest(&d);
        }
    }
    #[test]
    fn foreign_namespaces_case_adjustments_and_attribute_identity() {
        let d = parse(
            "<svg id=s VIEWBOX='0 0 10 10' xmlns='wrong' xml:lang=en xml:base=/ xlink:href='#a'><linearGradient id=g gradientUnits=userSpaceOnUse /><foreignObject id=f><DIV ID=h>HTML</DIV></foreignObject></svg><math id=m definitionurl=foo><mi id=i><mglyph id=glyph /><span id=span>text</span></mi></math>",
        );
        for (selector, namespace, tag) in [
            ("#s", Namespace::Svg, "svg"),
            ("#g", Namespace::Svg, "linearGradient"),
            ("#f", Namespace::Svg, "foreignObject"),
            ("#h", Namespace::Html, "div"),
            ("#m", Namespace::MathMl, "math"),
            ("#i", Namespace::MathMl, "mi"),
            ("#glyph", Namespace::MathMl, "mglyph"),
            ("#span", Namespace::Html, "span"),
        ] {
            let id = d.query_selector(selector).unwrap();
            assert_eq!(d.namespace(id), Some(namespace), "{selector}");
            assert_eq!(d.tag(id), Some(tag));
        }
        let svg = d.query_selector("#s").unwrap();
        assert_eq!(d.attr(svg, "viewBox"), Some("0 0 10 10"));
        assert_eq!(d.attr(svg, "viewbox"), None);
        let NodeKind::Element(element) = &d.nodes[svg].kind else {
            unreachable!()
        };
        assert_eq!(
            element.attr_namespaces.get("xmlns"),
            Some(&AttributeNamespace::Xmlns)
        );
        assert_eq!(
            element.attr_namespaces.get("xml:lang"),
            Some(&AttributeNamespace::Xml)
        );
        assert_eq!(
            element.attr_namespaces.get("xlink:href"),
            Some(&AttributeNamespace::XLink)
        );
        assert!(!element.attr_namespaces.contains_key("xml:base"));
        assert_eq!(
            d.attr(d.query_selector("#m").unwrap(), "definitionURL"),
            Some("foo")
        );
        assert!(d.query_selector("linearGradient[gradientUnits]").is_some());
        assert!(d.query_selector("lineargradient").is_none());
        assert!(d.query_selector("linearGradient[gradientunits]").is_none());
        assert_bounded_forest(&d);
    }
    #[test]
    fn mathml_and_svg_integration_points_dispatch_by_namespace_and_token_kind() {
        let d = parse(
            "<math><mtext><mglyph id=mg /><malignmark id=mark /><x id=x></x></mtext><annotation-xml id=a encoding='APPLICATION/XHTML+XML'><p id=p>html</p></annotation-xml><annotation-xml id=b><svg id=s><desc><em id=e>html</em></desc><title><b id=t>html</b></title></svg></annotation-xml><annotation-xml id=c encoding=' text/html'><x id=foreign /></annotation-xml></math>",
        );
        for (selector, namespace) in [
            ("#mg", Namespace::MathMl),
            ("#mark", Namespace::MathMl),
            ("#x", Namespace::Html),
            ("#a", Namespace::MathMl),
            ("#p", Namespace::Html),
            ("#b", Namespace::MathMl),
            ("#s", Namespace::Svg),
            ("#e", Namespace::Html),
            ("#t", Namespace::Html),
            ("#c", Namespace::MathMl),
            ("#foreign", Namespace::MathMl),
        ] {
            assert_eq!(
                d.namespace(d.query_selector(selector).unwrap()),
                Some(namespace),
                "{selector}"
            );
        }
        assert_eq!(d.title(), "");
        assert_bounded_forest(&d);
    }
    #[test]
    fn foreign_breakout_and_case_insensitive_end_tags_reprocess_in_html_mode() {
        let d = parse(
            "<svg id=s><g><font id=f>foreign</font><font color=red id=h>html</font><p id=p>p</svg><math><mrow></p><span id=after>after",
        );
        let svg = d.query_selector("#s").unwrap();
        let body = d.query_selector("body").unwrap();
        assert_eq!(
            d.namespace(d.query_selector("#f").unwrap()),
            Some(Namespace::Svg)
        );
        for selector in ["#h", "#p", "#after"] {
            let id = d.query_selector(selector).unwrap();
            assert_eq!(d.namespace(id), Some(Namespace::Html));
        }
        assert_eq!(d.nodes[svg].parent, Some(body));
        assert_eq!(d.nodes[d.query_selector("#h").unwrap()].parent, Some(body));
        let d = parse(
            "<svg><linearGradient><stop /></LINEARGRADIENT><path id=path /></svg><p id=outside>outside",
        );
        let path = d.query_selector("#path").unwrap();
        assert_eq!(d.tag(d.nodes[path].parent.unwrap()), Some("svg"));
        assert_eq!(
            d.namespace(d.query_selector("#outside").unwrap()),
            Some(Namespace::Html)
        );
    }
    #[test]
    fn foreign_cdata_nuls_and_script_text_do_not_use_html_raw_text_rules() {
        let d = parse(
            "<svg><g id=g><![CDATA[<&amp;\0]]>\0</g><script id=s><x />a &lt; b</script><title id=t><![CDATA[a\0b]]><b>bold</b></title></svg>",
        );
        assert_eq!(
            d.text_content(d.query_selector("#g").unwrap()),
            "<&amp;\u{fffd}\u{fffd}"
        );
        let script = d.query_selector("#s").unwrap();
        assert_eq!(
            d.namespace(d.nodes[script].children[0]),
            Some(Namespace::Svg)
        );
        assert_eq!(d.text_content(script), "a < b");
        assert_eq!(
            d.outer_html(script),
            "<script id=\"s\"><x></x>a &lt; b</script>"
        );
        assert_eq!(d.text_content(d.query_selector("#t").unwrap()), "abbold");
        assert!(d.query_selector("title > b").is_some());
    }
    #[test]
    fn namespaced_dom_mutation_and_snapshots_preserve_byte_accounting() {
        let mut d = parse("<div xlink:href=html></div>");
        let svg = d.create_element_ns(Namespace::Svg, "linearGradient");
        d.append_child(d.query_selector("body").unwrap(), svg);
        d.set_attr(svg, "viewBox", "0 0 1 1");
        d.set_attr_ns(svg, AttributeNamespace::XLink, "xlink:href", "#one");
        d.set_attr(svg, "xlink:href", "#two");
        let before = d.retained_bytes();
        assert_eq!(d.attr(svg, "viewBox"), Some("0 0 1 1"));
        assert_eq!(d.attr(svg, "viewbox"), None);
        assert!(
            Document::from_snapshot(d.nodes.clone(), d.root, false, DocumentMode::NoQuirks).is_ok()
        );
        d.remove_attr(svg, "xlink:href");
        assert_eq!(
            d.retained_bytes(),
            before - "xlink:href".len() * 2 - "#two".len()
        );
        let rebuilt =
            Document::from_snapshot(d.nodes.clone(), d.root, false, DocumentMode::NoQuirks)
                .unwrap();
        assert_eq!(rebuilt.retained_bytes(), d.retained_bytes());
        let mut invalid = d.nodes;
        let NodeKind::Element(element) = &mut invalid[svg].kind else {
            unreachable!()
        };
        element
            .attr_namespaces
            .insert("viewBox".into(), AttributeNamespace::Xml);
        assert!(Document::from_snapshot(invalid, d.root, false, DocumentMode::NoQuirks).is_err());
    }
    #[test]
    fn mixed_namespace_boundaries_remain_bounded_and_cycle_free() {
        for pattern in [
            "<svg><foreignObject><math><mtext>",
            "<b><svg><desc><p></b>",
            "<math><annotation-xml><svg><g></p>",
        ] {
            let d = parse(&pattern.repeat(2000));
            assert_bounded_forest(&d);
            assert!(
                Document::from_snapshot(d.nodes.clone(), d.root, false, DocumentMode::NoQuirks)
                    .is_ok()
            );
        }
    }
    #[test]
    fn document_metadata_excludes_only_html_template_content_and_foreign_lookalikes() {
        let mut d = parse(
            "<template><title>inert title</title><style>inert css</style></template><svg><title>svg title</title><style>svg css</style><template><foreignObject><title>active title</title><style>html css</style></foreignObject></template></svg><math><style>math text</style></math>",
        );
        assert_eq!(d.title(), "active title");
        assert_eq!(d.stylesheets(), ["svg css", "html css"]);
        let templates = d.query_selector_all("template");
        assert_eq!(templates.len(), 2);
        assert!(!d.is_active_node(templates[0]));
        assert!(d.is_active_node(templates[1]));
        let detached = d.create_element("style");
        d.set_text_content(detached, "detached css");
        assert!(!d.is_active_node(detached));
        assert_eq!(d.stylesheets(), ["svg css", "html css"]);
    }
    #[test]
    fn selector_control_states_language_and_sibling_types_keep_namespace_identity() {
        let mut d = parse(
            "<svg xml:lang=fr-CA><input id=foreign checked disabled required /><g id=language /></svg><input id=html checked disabled required>",
        );
        for selector in ["input:checked", "input:disabled", "input:required"] {
            assert_eq!(
                d.query_selector_all(selector),
                vec![d.query_selector("#html").unwrap()]
            );
        }
        assert!(d.query_selector("#foreign:enabled").is_none());
        assert!(d.query_selector("#foreign:optional").is_none());
        assert!(d.query_selector("#language:lang(fr)").is_some());
        let body = d.query_selector("body").unwrap();
        let foreign = d.create_element_ns(Namespace::MathMl, "input");
        d.append_child(body, foreign);
        assert!(matches_selector(&d, foreign, "input:first-of-type"));
        assert!(matches_selector(&d, foreign, "input:only-of-type"));
    }
    #[test]
    fn snapshot_validation_accepts_detached_forests_and_recomputes_bytes() {
        let mut original = parse_with_scripting(
            "<!doctype html><!--before--><p><b>one<i>two</b>three</i></p><?step done>",
            true,
        );
        let paragraph = original.query_selector("p").unwrap();
        let parent = original.nodes[paragraph].parent.unwrap();
        original.remove_child(parent, paragraph);
        let expected_bytes = original.retained_bytes();
        let rebuilt =
            Document::from_snapshot(original.nodes, original.root, true, DocumentMode::NoQuirks)
                .unwrap();
        assert_eq!(rebuilt.retained_bytes(), expected_bytes);
        assert!(rebuilt.scripting_enabled());
        assert_eq!(rebuilt.text_content(paragraph), "onetwothree");
        assert!(rebuilt.query_selector("p").is_none());
    }
    #[test]
    fn snapshot_validation_rejects_cycles_duplicate_edges_and_non_container_children() {
        let original = parse("<p>x</p>");
        let paragraph = original.query_selector("p").unwrap();
        let parent = original.nodes[paragraph].parent.unwrap();
        let text = original.nodes[paragraph].children[0];
        let mut duplicate = original.nodes.clone();
        duplicate[parent].children.push(paragraph);
        assert!(
            Document::from_snapshot(duplicate, original.root, false, DocumentMode::NoQuirks)
                .is_err()
        );
        let mut dangling = original.nodes.clone();
        dangling[paragraph].parent = Some(usize::MAX);
        assert!(
            Document::from_snapshot(dangling, original.root, false, DocumentMode::NoQuirks)
                .is_err()
        );
        let mut leaf = original.nodes.clone();
        leaf[paragraph].kind = NodeKind::Comment("forged".into());
        assert!(
            Document::from_snapshot(leaf, original.root, false, DocumentMode::NoQuirks).is_err()
        );
        let mut cycle = original.nodes.clone();
        cycle[parent].children.retain(|id| *id != paragraph);
        cycle[paragraph].parent = Some(text);
        cycle[text].kind = NodeKind::Element(Element {
            namespace: Namespace::Html,
            tag: "span".into(),
            attrs: BTreeMap::new(),
            attr_namespaces: BTreeMap::new(),
            template_contents: None,
        });
        cycle[text].children.push(paragraph);
        assert!(
            Document::from_snapshot(cycle, original.root, false, DocumentMode::NoQuirks).is_err()
        );
        let mut doctype = original.nodes;
        doctype[text].kind = NodeKind::Doctype(Doctype {
            name: "html".into(),
            public_id: None,
            system_id: None,
            force_quirks: false,
        });
        assert!(
            Document::from_snapshot(doctype, original.root, false, DocumentMode::NoQuirks).is_err()
        );
    }
    #[test]
    fn snapshot_validation_enforces_attribute_and_detached_depth_limits() {
        let mut original = parse("<p>x</p>");
        let paragraph = original.query_selector("p").unwrap();
        for index in 0..1024 {
            original.set_attr(paragraph, &format!("a{index}"), "");
        }
        assert!(
            Document::from_snapshot(
                original.nodes.clone(),
                original.root,
                false,
                DocumentMode::NoQuirks
            )
            .is_ok()
        );
        let NodeKind::Element(element) = &mut original.nodes[paragraph].kind else {
            unreachable!()
        };
        element.attrs.insert("extra".into(), String::new());
        assert!(
            Document::from_snapshot(original.nodes, original.root, false, DocumentMode::NoQuirks)
                .is_err()
        );
        let mut nodes = vec![Node {
            parent: None,
            children: vec![],
            kind: NodeKind::Document,
        }];
        for index in 1..=MAX_DEPTH + 2 {
            nodes.push(Node {
                parent: if index == 1 { None } else { Some(index - 1) },
                children: if index == MAX_DEPTH + 2 {
                    vec![]
                } else {
                    vec![index + 1]
                },
                kind: NodeKind::Element(Element {
                    namespace: Namespace::Html,
                    tag: "div".into(),
                    attrs: BTreeMap::new(),
                    attr_namespaces: BTreeMap::new(),
                    template_contents: None,
                }),
            });
        }
        assert!(Document::from_snapshot(nodes, 0, false, DocumentMode::NoQuirks).is_err());
    }
    fn assert_bounded_forest(document: &Document) {
        assert!(document.nodes.len() <= MAX_NODES);
        assert!(document.retained_bytes() <= MAX_DOM_BYTES);
        let mut seen = vec![false; document.nodes.len()];
        let mut pending: Vec<_> = document
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.parent.is_none()
                    && !matches!(node.kind, NodeKind::DocumentFragment { host: Some(_) })
            })
            .map(|(id, _)| (id, 1usize))
            .collect();
        while let Some((id, depth)) = pending.pop() {
            assert!(!seen[id], "node {id} occurs twice or a cycle exists");
            seen[id] = true;
            assert!(depth <= MAX_DEPTH, "node {id} at depth {depth}");
            for child in &document.nodes[id].children {
                assert_eq!(document.nodes[*child].parent, Some(id));
                pending.push((*child, depth + 1));
            }
            if let Some(content) = document.template_contents(id) {
                assert!(
                    matches!(document.nodes[content].kind, NodeKind::DocumentFragment { host: Some(host) } if host == id)
                );
                assert!(document.nodes[content].parent.is_none());
                pending.push((content, depth + 1));
            }
        }
        assert!(
            seen.into_iter().all(|visited| visited),
            "cycle detached from every root"
        );
    }
    #[test]
    fn template_contents_are_separate_inert_fragment_trees() {
        let d = Document::parse(
            "<!doctype html><template id=t><!--inside--><div id=x>text<template id=n><b id=y>nested</b></template></div></template><p id=active>live</p>",
        );
        let template = d.query_selector("#t").unwrap();
        let content = d.template_contents(template).unwrap();
        assert!(d.nodes[template].children.is_empty());
        assert!(d.nodes[content].parent.is_none());
        assert!(
            matches!(d.nodes[content].kind, NodeKind::DocumentFragment { host: Some(host) } if host == template)
        );
        assert!(d.query_selector("#x").is_none());
        assert!(d.query_selector("#n").is_none());
        let child = d.query_selector_from(content, "#x").unwrap();
        let nested = d.query_selector_from(content, "#n").unwrap();
        assert!(d.query_selector_from(content, "#y").is_none());
        assert!(!d.is_active_node(child));
        assert_eq!(d.text_content(template), "");
        assert_eq!(d.text_content(content), "text");
        assert_eq!(
            d.text_content(d.template_contents(nested).unwrap()),
            "nested"
        );
        assert!(
            d.outer_html(template)
                .contains("<template id=\"n\"><b id=\"y\">nested</b></template>")
        );
        assert_bounded_forest(&d);
        assert!(Document::from_snapshot(d.nodes.clone(), d.root, false, d.mode()).is_ok());
    }
    #[test]
    fn template_insertion_modes_recover_tables_forms_and_foreign_content() {
        for (inside, expected) in [
            ("<tr><td>cell", "<tr><td>cell</td></tr>"),
            ("<td>cell", "<td>cell</td>"),
            ("<col><col>", "<col><col>"),
            ("<tbody><tr><td>x", "<tbody><tr><td>x</td></tr></tbody>"),
            (
                "<table>foster<tr><td>x</table>",
                "foster<table><tbody><tr><td>x</td></tr></tbody></table>",
            ),
            (
                "<form id=a><form id=b>x</form>y</form>",
                "<form id=\"a\"><form id=\"b\">x</form>y</form>",
            ),
        ] {
            let d = Document::parse(&format!(
                "<form id=outer><template>{inside}</template><input></form>"
            ));
            let t = d.query_selector("template").unwrap();
            assert_eq!(
                d.outer_html(d.template_contents(t).unwrap()),
                expected,
                "{inside}"
            );
            assert_eq!(
                d.nodes[d.query_selector("input").unwrap()].parent,
                d.query_selector("#outer")
            );
            assert_bounded_forest(&d);
        }
        let d = Document::parse(
            "<template><svg><foreignObject><template><math><mi>x</mi></math></template></foreignObject></svg></template>",
        );
        let outer = d
            .template_contents(d.query_selector("template").unwrap())
            .unwrap();
        let inner = d.query_selector_from(outer, "template").unwrap();
        let content = d.template_contents(inner).unwrap();
        assert_eq!(
            d.namespace(d.query_selector_from(content, "mi").unwrap()),
            Some(Namespace::MathMl)
        );
        assert_bounded_forest(&d);
    }
    #[test]
    fn template_fragments_initialize_the_template_mode_stack() {
        let mut owner = Document::parse("<!doctype html>");
        let template = owner.create_element("template");
        let content = owner.template_contents(template).unwrap();
        for context in [template, content] {
            let d = owner
                .parse_fragment(context, "<tr><td>x</td></tr><template><col></template>")
                .unwrap();
            assert_eq!(
                d.outer_html(d.root),
                "<tr><td>x</td></tr><template><col></template>"
            );
            assert_eq!(d.mode(), DocumentMode::NoQuirks);
            assert_bounded_forest(&d);
        }
        assert!(owner.nodes[content].children.is_empty());
        for (source, expected) in [
            (
                "<form id=a><form id=b>x</form>y</form>",
                "<form id=\"a\"><form id=\"b\">x</form>y</form>",
            ),
            ("<form><div></form>after", "<form><div></div></form>after"),
        ] {
            let fragment = owner.parse_fragment(template, source).unwrap();
            assert_eq!(fragment.outer_html(fragment.root), expected);
            assert_bounded_forest(&fragment);
        }
    }
    #[test]
    fn template_recovery_preserves_document_frameset_and_attribute_state() {
        let d = Document::parse("<body><template><frameset><frame><p>text</template><p>active");
        let content = d
            .template_contents(d.query_selector("template").unwrap())
            .unwrap();
        assert_eq!(d.outer_html(content), "<p>text</p>");
        assert!(d.query_selector("frameset").is_none());
        let d = Document::parse("<head><template></template></head><p><frameset><frame>");
        assert!(d.query_selector("body").is_none());
        assert!(d.query_selector("html > frameset > frame").is_some());
        let d = Document::parse("<template><col><html data-leak=yes></template>");
        assert_eq!(d.attr(d.query_selector("html").unwrap(), "data-leak"), None);
        let d = Document::parse("<template><table><form id=a><tr><td>cell</table></template>");
        let content = d
            .template_contents(d.query_selector("template").unwrap())
            .unwrap();
        assert_eq!(
            d.outer_html(content),
            "<table><form id=\"a\"></form><tbody><tr><td>cell</td></tr></tbody></table>"
        );
        assert_bounded_forest(&d);
    }
    #[test]
    fn encoding_hints_follow_accepted_tokens_and_fragments_inherit_encoding() {
        let mut d = Document::parse(
            "<meta charset=invalid><svg><![CDATA[<meta charset=shift_jis>]]></svg><style><meta charset=windows-1251></style><meta charset=windows-1252><meta charset=utf-8><div id=context></div>",
        );
        assert_eq!(d.encoding_declaration(), Some(encoding_rs::WINDOWS_1252));
        assert_eq!(d.character_set(), "UTF-8");
        d.set_encoding(encoding_rs::WINDOWS_1251);
        let fragment = d
            .parse_fragment(
                d.query_selector("#context").unwrap(),
                "<meta charset=utf-8>",
            )
            .unwrap();
        assert_eq!(fragment.character_set(), "windows-1251");
        assert_eq!(fragment.encoding_declaration(), None);
        let declaration = Document::parse(
            "<meta content='text/html;charset=shift_jis'><meta http-equiv=CONTENT-TYPE content='text/html;charset=windows-1252'>",
        );
        assert_eq!(
            declaration.encoding_declaration(),
            Some(encoding_rs::WINDOWS_1252)
        );
        // Template contents are inert, but their meta tokens still use the
        // parser's in-head encoding-change rules before author execution.
        let declaration =
            Document::parse("<template><meta charset=windows-1251></template><meta charset=utf-8>");
        assert_eq!(
            declaration.encoding_declaration(),
            Some(encoding_rs::WINDOWS_1251)
        );
    }
    #[test]
    fn compatibility_modes_classify_doctypes_and_are_inherited_by_fragments() {
        for (doctype, expected) in [
            ("", DocumentMode::Quirks),
            ("<!doctype html>", DocumentMode::NoQuirks),
            (
                "<!doctype html system 'about:legacy-compat'>",
                DocumentMode::NoQuirks,
            ),
            ("<!doctype svg>", DocumentMode::Quirks),
            (
                "<!doctype html PUBLIC '-//W3C//DTD HTML 4.01 Transitional//EN'>",
                DocumentMode::Quirks,
            ),
            (
                "<!doctype html PUBLIC '-//W3C//DTD HTML 4.01 Transitional//EN' ''>",
                DocumentMode::Quirks,
            ),
            (
                "<!doctype html PUBLIC '-//W3C//DTD HTML 4.01 Transitional//EN' 'legacy.dtd'>",
                DocumentMode::LimitedQuirks,
            ),
            (
                "<!doctype html PUBLIC '-//w3c//dtd xhtml 1.0 frameset//EN'>",
                DocumentMode::LimitedQuirks,
            ),
            (
                "<!doctype html PUBLIC '-//W3C//DTD HTML 3.2 Final//EN'>",
                DocumentMode::Quirks,
            ),
            (
                "<!doctype html system 'HTTP://WWW.IBM.COM/DATA/DTD/V11/IBMXHTML1-TRANSITIONAL.DTD'>",
                DocumentMode::Quirks,
            ),
            ("<!doctype html PUBLIC 'unknown'>", DocumentMode::NoQuirks),
        ] {
            let owner = Document::parse(&format!("{doctype}<div id=context></div>"));
            assert_eq!(owner.mode(), expected, "{doctype}");
            let d = owner
                .parse_fragment(
                    owner.query_selector("#context").unwrap(),
                    "<p><table></table>",
                )
                .unwrap();
            assert_eq!(d.mode(), expected);
            let table = d.query_selector("table").unwrap();
            assert_eq!(
                d.tag(d.nodes[table].parent.unwrap()) == Some("p"),
                expected == DocumentMode::Quirks,
                "{doctype}"
            );
        }
        let owner = Document::parse("<template><div></div></template>");
        let content = owner
            .template_contents(owner.query_selector("template").unwrap())
            .unwrap();
        let context = owner.query_selector_from(content, "div").unwrap();
        assert_eq!(
            owner.parse_fragment(context, "<p><table>").unwrap().mode(),
            DocumentMode::NoQuirks
        );
    }
    #[test]
    fn fragment_transfer_and_template_host_cycles_preserve_dom_invariants() {
        let mut d = Document::parse("<body><template><div><template></template></div></template>");
        let body = d.query_selector("body").unwrap();
        let outer = d.query_selector("template").unwrap();
        let content = d.template_contents(outer).unwrap();
        let inner = d.query_selector_from(content, "template").unwrap();
        let inner_content = d.template_contents(inner).unwrap();
        let before = d.outer_html(d.root);
        d.append_child(inner_content, outer);
        d.append_child(inner_content, content);
        assert_eq!(d.outer_html(d.root), before);
        d.append_child(body, content);
        assert!(d.nodes[content].children.is_empty());
        assert!(d.nodes[content].parent.is_none());
        assert!(d.query_selector("body > div > template").is_some());
        assert!(d.template_contents(outer).is_some());
        let normal = d.create_element("i");
        d.append_child(outer, normal);
        assert_eq!(d.nodes[outer].children, vec![normal]);
        assert!(d.outer_html(outer).ends_with("></template>"));
        assert_bounded_forest(&d);
        assert!(Document::from_snapshot(d.nodes.clone(), d.root, false, d.mode()).is_ok());
    }
    #[test]
    fn snapshot_rejects_template_host_aliases_and_host_including_cycles() {
        let d = Document::parse("<template><template></template></template>");
        let outer = d.query_selector("template").unwrap();
        let content = d.template_contents(outer).unwrap();
        let inner = d.query_selector_from(content, "template").unwrap();
        let inner_content = d.template_contents(inner).unwrap();
        let mut wrong_host = d.nodes.clone();
        wrong_host[content].kind = NodeKind::DocumentFragment { host: Some(inner) };
        assert!(Document::from_snapshot(wrong_host, d.root, false, d.mode()).is_err());
        let mut missing = d.nodes.clone();
        if let NodeKind::Element(el) = &mut missing[outer].kind {
            el.template_contents = None;
        }
        assert!(Document::from_snapshot(missing, d.root, false, d.mode()).is_err());
        let mut cycle = d.nodes.clone();
        let old_parent = cycle[outer].parent.unwrap();
        cycle[old_parent].children.retain(|id| *id != outer);
        cycle[outer].parent = Some(inner_content);
        cycle[inner_content].children.push(outer);
        assert!(Document::from_snapshot(cycle, d.root, false, d.mode()).is_err());
        let mut attached_fragment = d.nodes.clone();
        attached_fragment[content].parent = Some(outer);
        attached_fragment[outer].children.push(content);
        assert!(Document::from_snapshot(attached_fragment, d.root, false, d.mode()).is_err());
    }
    #[test]
    fn adversarial_nested_template_recovery_obeys_host_depth_and_node_budgets() {
        for source in [
            "<template><table><b><tr><td><template>".repeat(1000),
            format!(
                "{}{}",
                "<template><b><div>".repeat(500),
                "</b></template>".repeat(500)
            ),
        ] {
            let d = Document::parse(&source);
            assert_bounded_forest(&d);
            assert!(Document::from_snapshot(d.nodes.clone(), d.root, false, d.mode()).is_ok());
        }
        let mut d = Document::parse("");
        while d.nodes.len() < MAX_NODES - 1 {
            d.create_comment("");
        }
        assert_eq!(d.create_element("temp!late"), d.root);
        assert!(Document::from_snapshot(d.nodes.clone(), d.root, false, d.mode()).is_ok());
    }
    #[test]
    fn formatting_reconstruction_reopens_misnested_inline_elements() {
        for (source, expected) in [
            (
                "<p>1<b>2<i>3</b>4</i>5",
                "<body><p>1<b>2<i>3</i></b><i>4</i>5</p></body>",
            ),
            (
                "<p><b>one</p>two<br>three",
                "<body><p><b>one</b></p><b>two<br>three</b></body>",
            ),
            (
                "<p>1<s id=A>2<b id=B>3</p>4</s>5</b>",
                "<body><p>1<s id=\"A\">2<b id=\"B\">3</b></s></p><s id=\"A\"><b id=\"B\">4</b></s><b id=\"B\">5</b></body>",
            ),
        ] {
            let d = parse(source);
            assert_eq!(
                d.outer_html(d.query_selector("body").unwrap()),
                expected,
                "{source}"
            );
            assert_bounded_forest(&d);
        }
    }
    #[test]
    fn adoption_reparents_blocks_and_preserves_bookmark_order() {
        for (source, expected) in [
            ("<b>1<p>2</b>3</p>", "<body><b>1</b><p><b>2</b>3</p></body>"),
            (
                "<a>1<button>2</a>3</button>",
                "<body><a>1</a><button><a>2</a>3</button></body>",
            ),
            (
                "<b><a><b><p></a>",
                "<body><b><a><b></b></a><b><p><a></a></p></b></b></body>",
            ),
            (
                "<a><b><b><p></a>",
                "<body><a><b><b></b></b></a><b><b><p><a></a></p></b></b></body>",
            ),
            (
                "<table><a>1<p>2</a>3</p>",
                "<body><a>1</a><p><a>2</a>3</p><table></table></body>",
            ),
        ] {
            let d = parse(source);
            assert_eq!(
                d.outer_html(d.query_selector("body").unwrap()),
                expected,
                "{source}"
            );
            assert_bounded_forest(&d);
        }
    }
    #[test]
    fn formatting_markers_stop_leaks_across_cells_and_objects() {
        for (source, expected) in [
            (
                "<p><b>before</p><table><tr><td>cell</td></tr></table>after",
                "<body><p><b>before</b></p><table><tbody><tr><td>cell</td></tr></tbody></table><b>after</b></body>",
            ),
            (
                "<p><b>one</p><object><i>two</object>three",
                "<body><p><b>one</b></p><b><object><i>two</i></object>three</b></body>",
            ),
            (
                "<table><a>1<td>2</td>3</table>",
                "<body><a>1</a><a>3</a><table><tbody><tr><td>2</td></tr></tbody></table></body>",
            ),
            (
                "<nobr><table><marquee></table><nobr>",
                "<body><nobr><marquee></marquee><table></table></nobr><nobr></nobr></body>",
            ),
        ] {
            let d = parse(source);
            assert_eq!(
                d.outer_html(d.query_selector("body").unwrap()),
                expected,
                "{source}"
            );
            assert_bounded_forest(&d);
        }
    }
    #[test]
    fn adoption_iteration_limits_retain_the_required_inner_and_outer_structure() {
        let d = parse("<div><a><b><u><i><code><div></a>");
        assert_eq!(
            d.outer_html(d.query_selector("body").unwrap()),
            "<body><div><a><b><u><i><code></code></i></u></b></a><u><i><code><div><a></a></div></code></i></u></div></body>"
        );
        assert_bounded_forest(&d);
        let d = parse(&format!("<div><a><b>{}</a>", "<div>".repeat(10)));
        assert_eq!(d.query_selector_all("a").len(), 9);
        assert_eq!(d.query_selector_all("b").len(), 2);
        assert_eq!(d.query_selector_all("a > div > div").len(), 1);
        assert_bounded_forest(&d);
    }
    #[test]
    fn noahs_ark_limits_identical_entries_without_losing_attributes() {
        let d =
            parse("<p><b id=x class=y><b class=y id=x><b id=x class=y><b class=y id=x>one</p>two");
        assert_eq!(d.query_selector_all("p b").len(), 4);
        assert_eq!(d.query_selector_all("body > b, body > b b").len(), 3);
        assert_eq!(d.query_selector_all("b#x.y").len(), 7);
        let d = parse("<a id=old>one<a id=new>two</a>three");
        assert_eq!(
            d.outer_html(d.query_selector("body").unwrap()),
            "<body><a id=\"old\">one</a><a id=\"new\">two</a>three</body>"
        );
        assert_bounded_forest(&d);
    }
    #[test]
    fn adoption_reparenting_rejects_cycles_and_excess_depth_before_mutation() {
        let mut builder = TreeBuilder::new(false);
        let mut parent = builder.doc.root;
        let mut first = 0;
        for _ in 1..MAX_DEPTH {
            let child = builder.doc.create_element("div");
            builder.doc.append_child(parent, child);
            if first == 0 {
                first = child;
            }
            parent = child;
        }
        let original = builder.doc.nodes[first].children.clone();
        let wrapper = builder.doc.create_element("b");
        assert!(!builder.wrap_children(first, wrapper));
        assert_eq!(builder.work, 0);
        assert_eq!(builder.doc.nodes[first].children, original);
        assert!(builder.doc.nodes[wrapper].parent.is_none());
        builder.work = 50_000_000;
        assert!(!builder.reparent(first, parent, None));
        assert_eq!(builder.doc.nodes[first].parent, Some(builder.doc.root));
        assert_bounded_forest(&builder.doc);
    }
    #[test]
    fn adoption_wraps_large_sibling_lists_with_linear_work_and_atomic_preflight() {
        let mut builder = TreeBuilder::new(false);
        let parent = builder.doc.create_element("div");
        builder.doc.append_child(builder.doc.root, parent);
        for _ in 0..10_000 {
            let child = builder.doc.create_element("span");
            builder.doc.append_child(parent, child);
        }
        let wrapper = builder.doc.create_element("b");
        builder.work = 20;
        assert!(!builder.wrap_children(parent, wrapper));
        assert_eq!(builder.doc.nodes[parent].children.len(), 10_000);
        assert!(builder.doc.nodes[wrapper].children.is_empty());
        builder.work = 30_000;
        assert!(builder.wrap_children(parent, wrapper));
        assert_eq!(builder.doc.nodes[parent].children, [wrapper]);
        assert_eq!(builder.doc.nodes[wrapper].children.len(), 10_000);
        assert_bounded_forest(&builder.doc);
    }
    #[test]
    fn reconstruction_and_reparenting_stop_at_shared_resource_limits() {
        let mut builder = TreeBuilder::new(false);
        for token in [
            HtmlToken::start("body"),
            HtmlToken::start("p"),
            HtmlToken::start("b"),
            HtmlToken::End("p".into()),
        ] {
            builder.process(token);
        }
        let old_len = builder.doc.nodes.len();
        builder.work = 0;
        builder.reconstruct_formatting(false);
        assert_eq!(builder.doc.nodes.len(), old_len);
        let d = parse(&format!(
            "{}tail",
            (0..3000)
                .map(|n| format!("<div><b data-n='{n}'>x</div>"))
                .collect::<String>()
        ));
        assert_bounded_forest(&d);
        let mut state = 0x92c314efu32;
        let tokens = [
            "<b>",
            "</b>",
            "<i>",
            "</i>",
            "<a>",
            "</a>",
            "<nobr>",
            "</nobr>",
            "<div>",
            "</div>",
            "<p>",
            "</p>",
            "<table>",
            "</table>",
            "<tr>",
            "<td>",
            "</td>",
            "<object>",
            "</object>",
            "x",
            " ",
        ];
        for _ in 0..120 {
            let mut source = String::new();
            for _ in 0..300 {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                source.push_str(tokens[state as usize % tokens.len()]);
            }
            assert_bounded_forest(&parse(&source));
        }
    }
    #[test]
    fn comments_doctypes_and_processing_instructions_keep_tree_positions() {
        let d = parse(
            "<!--before--><!DOCTYPE HTML PUBLIC 'public' 'system'><?build version?><html><!--html--><head><!--head--></head><!--between--><body>x<!--body--><?step done?></body><!--afterbody--></html><!--afterhtml-->",
        );
        assert_eq!(
            d.outer_html(d.root),
            "<!--before--><!DOCTYPE html><?build version?><html><!--html--><head><!--head--></head><!--between--><body>x<!--body--><?step done?></body><!--afterbody--></html><!--afterhtml-->"
        );
        assert_eq!(d.text_content(d.root), "x");
        assert!(
            matches!(&d.nodes[d.nodes[d.root].children[1]].kind, NodeKind::Doctype(Doctype { name, public_id: Some(public), system_id: Some(system), force_quirks: false }) if name == "html" && public == "public" && system == "system")
        );
    }
    #[test]
    fn malformed_comments_recover_without_swallowing_following_nodes() {
        for (source, expected) in [
            ("<!-->x", ""),
            ("<!--->x", ""),
            ("<!--a--!>x", "a"),
            ("<!--a<!--b-->x", "a<!--b"),
            ("<!--a\0-->x", "a�"),
            ("<!bogus>x", "bogus"),
            ("<?xml version='1'?>x", "?xml version='1'?"),
            ("<!--a--", "a"),
        ] {
            let d = parse(source);
            let comment = d
                .nodes
                .iter()
                .find_map(|node| match &node.kind {
                    NodeKind::Comment(data) => data.scalar(),
                    _ => None,
                })
                .unwrap();
            assert_eq!(comment, expected, "{source:?}");
            if source.ends_with('x') {
                assert_eq!(d.text_content(d.query_selector("body").unwrap()), "x");
            }
        }
    }
    #[test]
    fn processing_instruction_eof_and_targets_follow_tokenizer_rules() {
        for input in [
            "<?",
            "<?start",
            "<?start?",
            "<?start ",
            "<?start data",
            "<?start ? ?",
        ] {
            let d = parse(input);
            assert_eq!(d.nodes.len(), 4, "{input:?}");
        }
        let d = parse("<?Build\t ? data ??><body><?_step?><!DOCTYPE ignored>");
        assert!(
            matches!(&d.nodes[1].kind, NodeKind::ProcessingInstruction { target, data } if target == "Build" && data == "? data ?")
        );
        assert_eq!(d.text_content(1), "? data ?");
        assert!(
            !d.nodes
                .iter()
                .any(|node| matches!(node.kind, NodeKind::Doctype(_)))
        );
    }
    #[test]
    fn processing_instruction_serialization_preserves_trailing_question_marks() {
        // HTML fragment serialization appends a separate ?> after the data.
        // Without its ?, reparsing silently consumes a data-ending ? instead.
        for (data, expected) in [
            ("", "<?Build ?>"),
            ("a & < \"b\"", "<?Build a & < \"b\"?>"),
            ("ending?", "<?Build ending??>"),
            ("🦀", "<?Build 🦀?>"),
        ] {
            let mut original = parse("<div></div>");
            let pi = original.create_processing_instruction("Build", data);
            let serialized = original.outer_html(pi);
            assert_eq!(serialized, expected);
            let restored = parse(&format!("<div>{serialized}</div>"));
            let parent = restored.query_selector("div").unwrap();
            assert_eq!(restored.nodes[parent].children.len(), 1);
            let child = restored.nodes[parent].children[0];
            assert!(
                matches!(&restored.nodes[child].kind, NodeKind::ProcessingInstruction { target, data: actual } if target == "Build" && actual == data),
                "{serialized:?}"
            );
        }
    }
    #[test]
    fn non_container_mutations_and_retained_character_data_are_bounded() {
        let mut d = parse("<div><!--x--><?step value></div>");
        let div = d.query_selector("div").unwrap();
        assert_eq!(d.query_selector("div:empty"), Some(div));
        let comment = d.nodes[div].children[0];
        let pi = d.nodes[div].children[1];
        let doctype = d.create_doctype(Doctype {
            name: "html".into(),
            public_id: None,
            system_id: None,
            force_quirks: false,
        });
        let text = d.create_text_node("child");
        for parent in [comment, pi, doctype] {
            d.append_child(parent, text);
            assert!(d.nodes[parent].children.is_empty());
        }
        d.append_child(div, doctype);
        assert!(d.nodes[doctype].parent.is_none());
        let before = d.retained_bytes();
        d.set_text_content(comment, "new comment");
        d.set_text_content(pi, "new data");
        d.set_text_content(doctype, "ignored");
        assert_eq!(
            d.retained_bytes(),
            before - "x".len() - "value".len() + "new comment".len() + "new data".len()
        );
        assert_eq!(d.text_content(div), "");
        assert_eq!(d.text_content(comment), "new comment");
        let payload = "🦀".repeat(MAX_TEXT / 4 + 1);
        let large = d.create_comment(&payload);
        assert_eq!(d.text_content(large).len(), MAX_TEXT);
        assert!(d.outer_html(d.root).len() <= MAX_TEXT);
    }
    #[test]
    fn table_foster_parenting_preserves_text_order_and_implies_wrappers() {
        let d = parse("before<table> alpha <div>beta</div> gamma <tr><td>A<td>B</table>after");
        let body = d.query_selector("body").unwrap();
        assert_eq!(
            d.outer_html(body),
            "<body>before alpha <div>beta</div> gamma <table><tbody><tr><td>A</td><td>B</td></tr></tbody></table>after</body>"
        );
        assert_eq!(d.query_selector_all("table > tbody > tr > td").len(), 2);
        let d = parse("<table> \n <tr> \t <td>x</table>");
        let table = d.query_selector("table").unwrap();
        assert!(
            matches!(&d.nodes[d.nodes[table].children[0]].kind, NodeKind::Text(text) if text == " \n ")
        );
        // The '<' before a non-tag is a separate tokenizer character token;
        // all pending table characters must still move as one run.
        let d = parse("<table> \n <3</table>");
        assert_eq!(
            d.outer_html(d.query_selector("body").unwrap()),
            "<body> \n &lt;3<table></table></body>"
        );
    }
    #[test]
    fn table_modes_close_cells_rows_captions_and_column_groups() {
        let d = parse("<table><caption>C<col class=c><tbody><td>A<tr><th>B<tfoot><td>C</table>");
        assert_eq!(
            d.outer_html(d.query_selector("table").unwrap()),
            "<table><caption>C</caption><colgroup><col class=\"c\"></colgroup><tbody><tr><td>A</td></tr><tr><th>B</th></tr></tbody><tfoot><tr><td>C</td></tr></tfoot></table>"
        );
        let d = parse(
            "<table><!--keep--><input type=HiDdEn><form id=f><input id=outside><tr><td>x</table>",
        );
        let table = d.query_selector("table").unwrap();
        assert!(matches!(
            d.nodes[d.nodes[table].children[0]].kind,
            NodeKind::Comment(_)
        ));
        assert_eq!(
            d.nodes[d.query_selector("input[type=HiDdEn]").unwrap()].parent,
            Some(table)
        );
        assert_eq!(d.nodes[d.query_selector("#f").unwrap()].parent, Some(table));
        assert_eq!(
            d.nodes[d.query_selector("#outside").unwrap()].parent,
            d.query_selector("body")
        );
    }
    #[test]
    fn nested_tables_and_stray_structural_end_tags_reprocess_safely() {
        let d = parse(
            "<table></body></caption></td><td>first<table><td>nested</table><td>second</table><table><table>",
        );
        assert_eq!(d.query_selector_all("body > table").len(), 3);
        assert_eq!(d.query_selector_all("td > table").len(), 1);
        assert_eq!(
            d.query_selector_all("body > table > tbody > tr > td").len(),
            2
        );
        let mut builder = TreeBuilder::new(false);
        builder.work = 1;
        builder.process(HtmlToken::start("td"));
        assert_eq!(builder.work, 0);
    }
    #[test]
    fn scripting_flag_raw_text_newlines_and_script_escape_states() {
        let input = "<head><noscript><meta name=x></noscript></head><body><noscript><b>fallback</b></noscript><pre>\r\none\rtwo</pre><textarea>\n&lt;x&gt;</textarea>";
        let disabled = Document::parse_with_scripting(input, false);
        let enabled = Document::parse_with_scripting(input, true);
        assert!(!disabled.scripting_enabled());
        assert!(enabled.scripting_enabled());
        assert_eq!(disabled.query_selector_all("meta").len(), 1);
        assert!(enabled.query_selector("meta").is_none());
        assert_eq!(disabled.query_selector_all("b").len(), 1);
        assert!(enabled.query_selector("b").is_none());
        assert_eq!(
            disabled.text_content(disabled.query_selector("pre").unwrap()),
            "one\ntwo"
        );
        assert_eq!(
            disabled.text_content(disabled.query_selector("textarea").unwrap()),
            "<x>"
        );
        let d = parse("<script><!--<script </script/");
        assert_eq!(
            d.text_content(d.query_selector("script").unwrap()),
            "<!--<script </script/"
        );
        let d = parse("<script><!--<script>x</script>--></script><p>after");
        assert_eq!(
            d.text_content(d.query_selector("script").unwrap()),
            "<!--<script>x</script>-->"
        );
        assert_eq!(d.text_content(d.query_selector("p").unwrap()), "after");
        let d = parse("<head></head><style>unclosed");
        assert_eq!(
            d.outer_html(d.root),
            "<html><head><style>unclosed</style></head><body></body></html>"
        );
        assert_eq!(
            enabled.outer_html(enabled.query_selector_all("noscript")[1]),
            "<noscript><b>fallback</b></noscript>"
        );
    }
    #[test]
    fn many_ignored_end_tags_use_iteration_and_node_limits_stop_comments() {
        let d = parse(&format!("{}<p>done", "</>".repeat(100_000)));
        assert_eq!(d.text_content(d.query_selector("p").unwrap()), "done");
        let d = parse(&"<!--x-->".repeat(MAX_NODES + 1));
        assert_eq!(d.nodes.len(), MAX_NODES);
        assert!(d.retained_bytes() <= MAX_DOM_BYTES);
    }
    #[test]
    fn serialization_escapes_markup_and_preserves_foreign_siblings() {
        let d = parse(
            "<svg viewBox='0 0 20 20'><g/><rect width='10'/><text>&amp;&lt;&nbsp;</text></svg><input value='&quot;&amp;&nbsp;'><script>a < b && c\u{a0}</script>",
        );
        let svg = d.query_selector("svg").unwrap();
        assert_eq!(d.nodes[svg].children.len(), 3);
        assert_eq!(
            d.outer_html(svg),
            "<svg viewBox=\"0 0 20 20\"><g></g><rect width=\"10\"></rect><text>&amp;&lt;&nbsp;</text></svg>"
        );
        assert_eq!(
            d.outer_html(d.query_selector("input").unwrap()),
            "<input value=\"&quot;&amp;&nbsp;\">"
        );
        assert_eq!(
            d.outer_html(d.query_selector("script").unwrap()),
            "<script>a < b && c\u{a0}</script>"
        );
    }
    #[test]
    fn all_named_reference_forms_and_attribute_ambiguity() {
        assert_eq!(
            decode_entities("&NotEqualTilde; &CounterClockwiseContourIntegral; &notit; &amp"),
            "≂̸ ∳ ¬it; &"
        );
        let d = parse("<p title='&notit; &amp=1 &amp;=2'>&#9999999999999999999999;</p>");
        let p = d.query_selector("p").unwrap();
        assert_eq!(d.attr(p, "title"), Some("&notit; &amp=1 &=2"));
        assert_eq!(d.text_content(p), "�");
    }
    #[test]
    fn selectors_exhaust_shared_work_budget_without_backtracking_explosion() {
        let d = parse(&format!(
            "{}<span>x</span>{}",
            "<div>".repeat(60),
            "</div>".repeat(60)
        ));
        let span = d.query_selector("span").unwrap();
        let mut budget = 2000;
        assert!(!matches_selector_with_budget(
            &d,
            span,
            &format!("missing {}span", "div ".repeat(20)),
            &mut budget
        ));
        assert_eq!(budget, 0);
        assert!(nth_matches("n-9223372036854775808", 2));
    }
    #[test]
    fn repeated_mutations_keep_retained_text_and_attributes_bounded() {
        let mut d = parse("<div></div>");
        let div = d.query_selector("div").unwrap();
        let payload = "a".repeat(1024 * 1024);
        for n in 0..40 {
            d.set_text_content(div, &payload);
            d.set_attr(div, &format!("data-{n}"), &payload);
        }
        assert!(d.retained_bytes() <= MAX_DOM_BYTES);
        let actual: usize = d
            .nodes
            .iter()
            .map(|n| match &n.kind {
                NodeKind::Text(s) | NodeKind::Comment(s) => s.stored_bytes(),
                NodeKind::Element(e) => e.retained_bytes(),
                NodeKind::Doctype(d) => {
                    d.name.len()
                        + d.public_id.as_ref().map_or(0, String::len)
                        + d.system_id.as_ref().map_or(0, String::len)
                }
                NodeKind::ProcessingInstruction { target, data } => {
                    target.len() + data.stored_bytes()
                }
                NodeKind::Document | NodeKind::DocumentFragment { .. } => 0,
            })
            .sum();
        assert_eq!(actual, d.retained_bytes());
    }
    #[test]
    fn malformed_unicode_inputs_do_not_panic() {
        let alphabet: Vec<char> = "<>/='\"&;#()[]:!not 01234é中🦀\\\n\0".chars().collect();
        let mut state = 0x12345678u32;
        for _ in 0..600 {
            let mut input = String::new();
            for _ in 0..160 {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                input.push(alphabet[state as usize % alphabet.len()]);
            }
            let d = parse(&input);
            let _ = d.query_selector_all(&input);
            let _ = crate::css::parse_stylesheet(&input, 800.0, 600.0);
        }
    }
    #[test]
    fn implied_structure_entities_and_rawtext() {
        let d = parse(
            "<!doctype html><title>A &amp; B</title><p id='x'>hello &lt;b&gt;<script>if (a < b) { x = '&amp;' }</script><p>next",
        );
        assert_eq!(d.title(), "A & B");
        assert_eq!(d.query_selector_all("body > p").len(), 2);
        assert!(
            d.text_content(d.query_selector("script").unwrap())
                .contains("a < b")
        );
        assert_eq!(
            d.text_content(d.query_selector("#x").unwrap()),
            "hello <b>if (a < b) { x = '&amp;' }"
        );
    }
    #[test]
    fn selector_comments_keep_actual_relationships_strings_and_nth_arithmetic() {
        let d = Document::parse(
            r#"<div id=own class=x data-a=Ab data-v="'"><span id=child class=x></span></div><p id=first></p><p id=second></p>"#,
        );
        let own = d.query_selector("#own").unwrap();
        let child = d.query_selector("#child").unwrap();
        assert_eq!(d.query_selector("div/**/.x"), Some(own));
        assert_eq!(d.query_selector("div /**/.x"), Some(child));
        assert_eq!(d.query_selector("div/**/>/**/./**/x"), Some(child));
        assert_eq!(d.query_selector(r#"[data-v="'"]"#), Some(own));
        assert_eq!(d.query_selector("[data-a=Ab/**/S]"), Some(own));
        assert_eq!(d.query_selector(":/**/IS(div.x)"), Some(own));
        assert_eq!(d.query_selector("div:FIRST-CHILD"), Some(own));
        for selector in [
            "div/**/span",
            "#/**/own",
            ":is/**/(div)",
            ":empty(div)",
            ":not(:madeup)",
            "[data-a~ /**/=Ab]",
        ] {
            assert!(d.query_selector(selector).is_none(), "{selector}");
        }
        let d = Document::parse("<p id=first></p><p></p>");
        assert_eq!(d.query_selector_all("p,:is()"), d.query_selector_all("p"));
        assert_eq!(
            d.query_selector_all("p:not(:is(:madeup))"),
            d.query_selector_all("p")
        );
        assert_eq!(
            d.query_selector("p:nth-child(3074457345618258603n-9223372036854775808)"),
            d.query_selector("#first")
        );
        assert_eq!(
            d.query_selector("p:nth-child(2n/**/-/**/1)"),
            d.query_selector("#first")
        );
        assert!(d.query_selector("p:nth-child(2/**/n-1)").is_none());
    }

    #[test]
    fn selectors_combinators_and_attributes() {
        let d = parse(
            "<div id=a><p class='x y' data-v='Hello world'>one</p><p>two <b>bold</b></p><p>three</p></div>",
        );
        assert_eq!(d.query_selector_all("#a > p:nth-child(2) b").len(), 1);
        assert_eq!(d.query_selector_all("p.x + p").len(), 1);
        assert_eq!(d.query_selector_all("p.x ~ p").len(), 2);
        assert_eq!(d.query_selector_all("[data-v^='hello' i]").len(), 1);
        assert_eq!(d.query_selector_all("p:not(.x):last-child").len(), 1);
    }
    #[test]
    fn bounded_and_cycle_free_mutation() {
        let mut d = parse("<div><p>x</p></div>");
        let div = d.query_selector("div").unwrap();
        let p = d.query_selector("p").unwrap();
        d.append_child(p, div);
        assert_ne!(d.nodes[div].parent, Some(p));
        d.set_text_content(div, "replacement");
        assert!(d.query_selector("p").is_none());
        assert_eq!(d.text_content(div), "replacement");
    }
    #[test]
    fn hostile_input_is_bounded() {
        let d = parse(&format!(
            "{}tail{}",
            "<div>".repeat(2000),
            "</div>".repeat(2000)
        ));
        let mut max = 0;
        for id in 0..d.nodes.len() {
            let mut at = Some(id);
            let mut depth = 0;
            while let Some(n) = at {
                depth += 1;
                at = d.nodes[n].parent;
            }
            max = max.max(depth);
        }
        assert!(max <= MAX_DEPTH);
        assert!(d.text_content(d.root).ends_with("tail"));
    }
    #[test]
    fn numeric_entities_recover_invalid_unicode() {
        assert_eq!(
            decode_entities("&#0; &#xD800; &#x80; &#128512;"),
            "� � € 😀"
        );
    }
    // Private accounting setup avoids allocating MAX_TEXT/MAX_DOM_BYTES payloads.

    #[test]
    fn character_data_preallocation_check_preserves_kind_and_size_boundaries() {
        let mut doc = Document::parse("<p></p>");
        let parent = doc.query_selector("p").unwrap();
        let text = doc
            .create_text_node_owned(DomString::from_nonscalar_units(vec![0xd800, 65]).unwrap())
            .unwrap();
        let comment = doc.create_comment("abcd");
        let pi = doc
            .create_processing_instruction_owned("probe".into(), "abcd".into())
            .unwrap();
        let count = doc.nodes.len();
        let capacity = doc.nodes.capacity();
        // Each existing payload is four bytes, despite differing encodings.
        doc.retained_bytes = MAX_DOM_BYTES;
        for id in [text, comment, pi] {
            assert_eq!(
                doc.check_character_data_replacement(id, 4),
                Ok(MAX_DOM_BYTES)
            );
            assert_eq!(
                doc.check_character_data_replacement(id, 5),
                Err(DomDataError::LimitExceeded)
            );
            assert_eq!(
                doc.check_character_data_replacement(id, 0),
                Ok(MAX_DOM_BYTES - 4)
            );
        }
        for id in [doc.root, parent, usize::MAX] {
            assert_eq!(
                doc.check_character_data_replacement(id, usize::MAX),
                Err(DomDataError::InvalidNode)
            );
        }
        assert_eq!(doc.retained_bytes, MAX_DOM_BYTES);
        doc.retained_bytes = 4;
        assert_eq!(
            doc.check_character_data_replacement(text, MAX_TEXT),
            Ok(MAX_TEXT)
        );
        for bytes in [MAX_TEXT + 1, usize::MAX] {
            assert_eq!(
                doc.check_character_data_replacement(text, bytes),
                Err(DomDataError::LimitExceeded)
            );
        }
        doc.retained_bytes = 3;
        assert_eq!(
            doc.check_character_data_replacement(text, 0),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(doc.retained_bytes, 3);
        assert_eq!((doc.nodes.len(), doc.nodes.capacity()), (count, capacity));
        assert!(matches!(&doc.nodes[text].kind, NodeKind::Text(data)
            if data.raw_units() == Some([0xd800, 65].as_slice())));
        assert!(matches!(&doc.nodes[comment].kind, NodeKind::Comment(data) if data == "abcd"));
        assert!(
            matches!(&doc.nodes[pi].kind, NodeKind::ProcessingInstruction { target, data }
            if target == "probe" && data == "abcd")
        );
    }

    #[test]
    fn character_data_publication_rechecks_after_an_earlier_admission() {
        let mut doc = Document::parse("<p></p>");
        let parent = doc.query_selector("p").unwrap();
        let text = doc
            .create_text_node_owned(DomString::from_nonscalar_units(vec![0xd800, 65]).unwrap())
            .unwrap();
        doc.append_child(parent, text);
        let children = doc.nodes[parent].children.clone();
        let count = doc.nodes.len();
        let capacity = doc.nodes.capacity();
        assert!(doc.check_character_data_replacement(text, 5).is_ok());
        // An earlier admission is not a reservation of future document space.
        doc.retained_bytes = MAX_DOM_BYTES;
        assert_eq!(
            doc.replace_character_data(text, "abcde".into()),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(doc.retained_bytes, MAX_DOM_BYTES);
        assert!(matches!(&doc.nodes[text].kind, NodeKind::Text(data)
            if data.raw_units() == Some([0xd800, 65].as_slice())));
        let scalar = "é".to_owned();
        let pointer = scalar.as_ptr();
        doc.replace_character_data(text, scalar.into()).unwrap();
        assert_eq!(doc.retained_bytes, MAX_DOM_BYTES - 2);
        assert!(matches!(&doc.nodes[text].kind, NodeKind::Text(data)
            if data.scalar().is_some_and(|value| value == "é" && value.as_ptr() == pointer)));
        assert_eq!((doc.nodes.len(), doc.nodes.capacity()), (count, capacity));
        assert_eq!(doc.nodes[text].parent, Some(parent));
        assert_eq!(doc.nodes[parent].children, children);
    }

    #[test]
    fn checked_character_data_preserves_owned_buffers_identity_and_snapshot_accounting() {
        let mut doc = Document::parse("<div></div>");
        let parent = doc.query_selector("div").unwrap();
        let target = "Build:🦀".to_owned();
        let data = "a\0🦀?".to_owned();
        let target_ptr = target.as_ptr();
        let data_ptr = data.as_ptr();
        let pi = doc
            .create_processing_instruction_owned(target, data.into())
            .unwrap();
        assert!(doc.nodes[pi].parent.is_none());
        let NodeKind::ProcessingInstruction { target, data } = &doc.nodes[pi].kind else {
            panic!("expected PI");
        };
        assert_eq!(target.as_ptr(), target_ptr);
        assert_eq!(data.scalar().unwrap().as_ptr(), data_ptr);
        assert_eq!(data, "a\0🦀?");
        let text = doc.create_text_node("old text");
        let comment = doc.create_comment("old comment");
        for id in [text, comment, pi] {
            doc.append_child(parent, id);
        }
        let children = doc.nodes[parent].children.clone();
        for id in [text, comment, pi] {
            let before = doc.retained_bytes;
            let old = doc.text_content(id).len();
            let replacement = "new\0🦀?>".to_owned();
            let ptr = replacement.as_ptr();
            let len = replacement.len();
            doc.replace_character_data(id, replacement.into()).unwrap();
            let stored = match &doc.nodes[id].kind {
                NodeKind::Text(data)
                | NodeKind::Comment(data)
                | NodeKind::ProcessingInstruction { data, .. } => data,
                _ => panic!("expected CharacterData"),
            };
            assert_eq!(stored.scalar().unwrap().as_ptr(), ptr);
            assert_eq!(stored, "new\0🦀?>");
            assert_eq!(doc.nodes[id].parent, Some(parent));
            assert!(doc.nodes[id].children.is_empty());
            assert_eq!(doc.retained_bytes, before - old + len);
        }
        assert_eq!(doc.nodes[parent].children, children);
        assert!(matches!(&doc.nodes[pi].kind,
            NodeKind::ProcessingInstruction { target, .. } if target == "Build:🦀"));
        let before = doc.retained_bytes;
        for id in [doc.root, parent, usize::MAX] {
            assert_eq!(
                doc.replace_character_data(id, "refused".into()),
                Err(DomDataError::InvalidNode)
            );
        }
        assert_eq!(doc.retained_bytes, before);
        let rebuilt =
            Document::from_snapshot(doc.nodes.clone(), doc.root, false, doc.mode).unwrap();
        assert_eq!(rebuilt.retained_bytes, doc.retained_bytes);
        assert_eq!(rebuilt.outer_html(parent), doc.outer_html(parent));
    }

    #[test]
    fn checked_processing_instruction_refuses_limits_without_truncation_or_root_alias() {
        let mut doc = Document::parse("");
        let count = doc.nodes.len();
        let bytes = doc.retained_bytes;
        let capacity = doc.nodes.capacity();
        assert_eq!(
            doc.create_processing_instruction_owned("x".into(), "a".repeat(MAX_TEXT).into()),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(
            (doc.nodes.len(), doc.retained_bytes, doc.nodes.capacity()),
            (count, bytes, capacity)
        );
        let pi = doc
            .create_processing_instruction_owned("x".into(), "a".repeat(MAX_TEXT - 1).into())
            .unwrap();
        assert_ne!(pi, doc.root);
        assert_eq!(doc.retained_bytes, bytes + MAX_TEXT);
        assert_eq!(doc.text_content(pi).len(), MAX_TEXT - 1);
        let before = doc.retained_bytes;
        assert_eq!(
            doc.replace_character_data(pi, "b".repeat(MAX_TEXT + 1).into()),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(doc.retained_bytes, before);
        assert_eq!(doc.text_content(pi).len(), MAX_TEXT - 1);
        doc.nodes.resize_with(MAX_NODES, || Node {
            parent: None,
            children: Vec::new(),
            kind: NodeKind::Text(DomString::default()),
        });
        assert_eq!(
            doc.create_processing_instruction_owned("x".into(), DomString::default()),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(doc.nodes.len(), MAX_NODES);
        assert_eq!(doc.retained_bytes, before);
        // Replacing an existing node needs no new node slot.
        doc.replace_character_data(pi, "short".into()).unwrap();
        assert_eq!(doc.text_content(pi), "short");
    }

    #[test]
    fn checked_character_data_replacement_reuses_and_releases_actual_retained_bytes() {
        let mut doc = Document::parse("");
        let pi = doc
            .create_processing_instruction_owned("x".into(), "old".into())
            .unwrap();
        while doc.retained_bytes < MAX_DOM_BYTES {
            let remaining = MAX_DOM_BYTES - doc.retained_bytes;
            let size = remaining.min(MAX_TEXT);
            let id = doc.create_comment(&"a".repeat(size));
            assert_ne!(id, doc.root);
        }
        assert_eq!(doc.retained_bytes, MAX_DOM_BYTES);
        doc.replace_character_data(pi, "new".into()).unwrap();
        assert_eq!(doc.retained_bytes, MAX_DOM_BYTES);
        assert_eq!(
            doc.replace_character_data(pi, "more".into()),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(doc.text_content(pi), "new");
        let count = doc.nodes.len();
        assert_eq!(
            doc.create_processing_instruction_owned("x".into(), DomString::default()),
            Err(DomDataError::LimitExceeded)
        );
        assert_eq!(doc.nodes.len(), count);
        doc.replace_character_data(pi, DomString::default())
            .unwrap();
        assert_eq!(doc.retained_bytes, MAX_DOM_BYTES - 3);
        let next = doc
            .create_processing_instruction_owned("x".into(), "é".into())
            .unwrap();
        assert_ne!(next, doc.root);
        assert_eq!(doc.retained_bytes, MAX_DOM_BYTES);
        let rebuilt = Document::from_snapshot(doc.nodes, doc.root, false, doc.mode).unwrap();
        assert_eq!(rebuilt.retained_bytes, MAX_DOM_BYTES);
    }

    #[test]
    fn script_text_admission_checks_per_node_and_remaining_byte_boundaries() {
        let mut doc = Document::parse("");
        let count = doc.nodes.len();
        let retained = doc.retained_bytes;
        assert!(doc.admits_text_node(MAX_TEXT));
        assert!(!doc.admits_text_node(MAX_TEXT + 1));
        assert!(!doc.admits_text_node(usize::MAX));
        assert_eq!(doc.nodes.len(), count);
        assert_eq!(doc.retained_bytes, retained);

        // A controlled near-full ledger tests the independent document-byte cap.
        // It represents earlier retained payloads without constructing them here.
        doc.retained_bytes = MAX_DOM_BYTES - 3;
        assert!(doc.admits_text_node(3));
        assert!(!doc.admits_text_node(4));
        assert_eq!(doc.retained_bytes, MAX_DOM_BYTES - 3);
        let text = doc.create_text_node("€");
        assert!(matches!(&doc.nodes[text].kind, NodeKind::Text(value) if value == "€"));
        assert_eq!(doc.nodes[text].parent, None);
        assert_eq!(doc.retained_bytes, MAX_DOM_BYTES);
        assert!(doc.admits_text_node(0));
        assert!(!doc.admits_text_node(1));
    }

    #[test]
    fn public_append_refuses_one_short_retained_storage_instead_of_truncating_text() {
        for strict in [false, true] {
            for remaining in [2, 1] {
                let mut doc = Document::parse("<body><i></i></body>");
                let body = doc.query_selector("body").unwrap();
                let children = doc.nodes[body].children.clone();
                let count = doc.nodes.len();
                // Only the ledger is near its limit; the fixture stays tiny.
                doc.retained_bytes = MAX_DOM_BYTES - remaining;
                let mut runtime = crate::script::Runtime::new();
                let source = "document.body.append('é');";
                let result = if strict {
                    runtime.execute_strict(source, &mut doc)
                } else {
                    runtime.execute(source, &mut doc)
                };
                if remaining == 2 {
                    assert_eq!(result.unwrap(), crate::script::Value::Undefined);
                    assert_eq!(doc.nodes.len(), count + 1);
                    assert_eq!(&doc.nodes[body].children[..children.len()], &children);
                    assert_eq!(doc.nodes[body].children.len(), children.len() + 1);
                    let appended = *doc.nodes[body].children.last().unwrap();
                    assert!(
                        matches!(&doc.nodes[appended].kind, NodeKind::Text(value) if value == "é")
                    );
                    assert_eq!(doc.nodes[appended].parent, Some(body));
                    assert_eq!(doc.retained_bytes, MAX_DOM_BYTES);
                } else {
                    assert!(result.unwrap_err().is_resource_limit());
                    assert_eq!(doc.nodes.len(), count);
                    assert_eq!(doc.nodes[body].children, children);
                    assert_eq!(doc.retained_bytes, MAX_DOM_BYTES - 1);
                }
            }
        }
    }
}
