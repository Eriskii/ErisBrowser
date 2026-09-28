//! A bounded, independent HTML tree builder and CSS selector matcher.
use std::collections::{BTreeMap, BTreeSet};

pub type NodeId = usize;
pub const MAX_NODES: usize = 100_000;
pub const MAX_DEPTH: usize = 256;
const MAX_TEXT: usize = 8 * 1024 * 1024;
pub const MAX_DOM_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub attrs: BTreeMap<String, String>,
}
#[derive(Debug, Clone)]
pub enum NodeKind {
    Document,
    Element(Element),
    Text(String),
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
}

impl Default for Document {
    fn default() -> Self {
        Self::parse("")
    }
}
impl Document {
    pub fn parse(source: &str) -> Self {
        parse(source)
    }
    pub fn attr(&self, id: NodeId, name: &str) -> Option<&str> {
        match &self.nodes.get(id)?.kind {
            NodeKind::Element(el) => el.attrs.get(&name.to_ascii_lowercase()).map(String::as_str),
            _ => None,
        }
    }
    pub fn tag(&self, id: NodeId) -> Option<&str> {
        match &self.nodes.get(id)?.kind {
            NodeKind::Element(el) => Some(&el.tag),
            _ => None,
        }
    }
    pub fn text_content(&self, id: NodeId) -> String {
        let mut result = String::new();
        let mut pending = vec![id];
        let mut visited = BTreeSet::new();
        while let Some(n) = pending.pop() {
            if !visited.insert(n) || visited.len() > MAX_NODES {
                continue;
            }
            let Some(node) = self.nodes.get(n) else {
                continue;
            };
            match &node.kind {
                NodeKind::Text(s) => {
                    let left = MAX_TEXT.saturating_sub(result.len());
                    result.push_str(&s[..floor_boundary(s, left.min(s.len()))]);
                }
                _ => pending.extend(node.children.iter().rev().copied()),
            }
        }
        result
    }
    pub fn set_text_content(&mut self, id: NodeId, text: &str) {
        if id >= self.nodes.len() {
            return;
        }
        let old_bytes = match &self.nodes[id].kind {
            NodeKind::Text(t) => t.len(),
            _ => 0,
        };
        let available = MAX_DOM_BYTES.saturating_sub(self.retained_bytes.saturating_sub(old_bytes));
        let text = &text[..floor_boundary(text, text.len().min(MAX_TEXT).min(available))];
        if let NodeKind::Text(t) = &mut self.nodes[id].kind {
            self.retained_bytes = self.retained_bytes.saturating_sub(t.len()) + text.len();
            *t = text.to_owned();
            return;
        }
        let children = std::mem::take(&mut self.nodes[id].children);
        for child in children {
            if let Some(n) = self.nodes.get_mut(child) {
                n.parent = None;
            }
        }
        if !text.is_empty() && self.nodes.len() < MAX_NODES {
            self.retained_bytes += text.len();
            let child = self.nodes.len();
            self.nodes.push(Node {
                parent: Some(id),
                children: vec![],
                kind: NodeKind::Text(text.to_owned()),
            });
            self.nodes[id].children.push(child);
        }
    }
    pub fn create_element(&mut self, tag: &str) -> NodeId {
        if self.nodes.len() >= MAX_NODES {
            return self.root;
        }
        let tag = tag
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | ':' | '_'))
            .take(256)
            .collect::<String>()
            .to_ascii_lowercase();
        if tag.len() > MAX_DOM_BYTES.saturating_sub(self.retained_bytes) {
            return self.root;
        }
        self.retained_bytes += tag.len();
        let id = self.nodes.len();
        self.nodes.push(Node {
            parent: None,
            children: vec![],
            kind: NodeKind::Element(Element {
                tag,
                attrs: BTreeMap::new(),
            }),
        });
        id
    }
    pub fn create_text_node(&mut self, text: &str) -> NodeId {
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
            kind: NodeKind::Text(text.into()),
        });
        id
    }
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        if parent >= self.nodes.len()
            || child >= self.nodes.len()
            || child == self.root
            || parent == child
            || matches!(self.nodes[parent].kind, NodeKind::Text(_))
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
            cursor = self.nodes.get(id).and_then(|n| n.parent);
        }
        let mut pending = vec![(child, 1usize)];
        let mut visited = 0;
        while let Some((id, depth)) = pending.pop() {
            visited += 1;
            if ancestors + depth > MAX_DEPTH || visited > MAX_NODES {
                return;
            }
            if let Some(n) = self.nodes.get(id) {
                pending.extend(n.children.iter().map(|n| (*n, depth + 1)));
            }
        }
        if let Some(old) = self.nodes[child].parent
            && let Some(n) = self.nodes.get_mut(old)
        {
            n.children.retain(|c| *c != child);
        }
        self.nodes[child].parent = Some(parent);
        self.nodes[parent].children.push(child);
    }
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        if self.nodes.get(child).and_then(|n| n.parent) == Some(parent) {
            self.nodes[parent].children.retain(|n| *n != child);
            self.nodes[child].parent = None;
        }
    }
    pub fn set_attr(&mut self, id: NodeId, name: &str, value: &str) {
        let name = name[..floor_boundary(name, name.len().min(1024))].to_ascii_lowercase();
        if let Some(Node {
            kind: NodeKind::Element(el),
            ..
        }) = self.nodes.get_mut(id)
        {
            if el.attrs.len() >= 1024 && !el.attrs.contains_key(&name) {
                return;
            }
            let old = el.attrs.get(&name);
            let old_size = old.map(|v| v.len() + name.len()).unwrap_or(0);
            let remaining =
                MAX_DOM_BYTES.saturating_sub(self.retained_bytes.saturating_sub(old_size));
            if remaining < name.len() {
                return;
            }
            let value = &value
                [..floor_boundary(value, value.len().min(MAX_TEXT).min(remaining - name.len()))];
            self.retained_bytes =
                self.retained_bytes.saturating_sub(old_size) + name.len() + value.len();
            el.attrs.insert(name, value.into());
        }
    }
    pub fn remove_attr(&mut self, id: NodeId, name: &str) {
        let name = name.to_ascii_lowercase();
        if let Some(Node {
            kind: NodeKind::Element(el),
            ..
        }) = self.nodes.get_mut(id)
            && let Some(old) = el.attrs.remove(&name)
        {
            self.retained_bytes = self.retained_bytes.saturating_sub(name.len() + old.len());
        }
    }
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
    pub fn query_selector(&self, selector: &str) -> Option<NodeId> {
        self.query_selector_impl(selector, true).into_iter().next()
    }
    pub fn query_selector_all(&self, selector: &str) -> Vec<NodeId> {
        self.query_selector_impl(selector, false)
    }
    fn query_selector_impl(&self, selector: &str, first: bool) -> Vec<NodeId> {
        let mut budget = 4_000_000usize;
        let mut result = vec![];
        let mut pending = vec![self.root];
        let mut visited = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !visited.insert(id) || visited.len() > MAX_NODES {
                continue;
            }
            if let Some(node) = self.nodes.get(id) {
                if matches!(node.kind, NodeKind::Element(_))
                    && matches_selector_with_budget(self, id, selector, &mut budget)
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
        self.query_selector_all("style")
            .into_iter()
            .map(|id| self.text_content(id))
            .collect()
    }
    pub fn title(&self) -> String {
        self.query_selector("title")
            .map(|id| self.text_content(id))
            .unwrap_or_default()
    }
    /// Serialize the attached subtree without recursion, retaining at most 8 MiB.
    pub fn outer_html(&self, id: NodeId) -> String {
        let mut out = String::new();
        let mut pending = vec![(id, false)];
        let mut visited = BTreeSet::new();
        while let Some((id, closing)) = pending.pop() {
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
                NodeKind::Document => {
                    pending.extend(node.children.iter().rev().map(|id| (*id, false)))
                }
                NodeKind::Text(text) => {
                    if node
                        .parent
                        .and_then(|parent| self.tag(parent))
                        .is_some_and(|tag| {
                            matches!(
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
                        push_serialized(&mut out, text);
                    } else {
                        escape_serialized(&mut out, text, false);
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
                    if !is_void(&element.tag) {
                        pending.push((id, true));
                        pending.extend(node.children.iter().rev().map(|id| (*id, false)));
                    }
                }
            }
        }
        out
    }
    fn push_text(&mut self, parent: NodeId, text: String) {
        let mut text = text;
        let available = MAX_DOM_BYTES.saturating_sub(self.retained_bytes);
        text.truncate(floor_boundary(&text, text.len().min(available)));
        if text.is_empty() {
            return;
        }
        self.retained_bytes += text.len();
        if let Some(last) = self.nodes[parent].children.last().copied()
            && let NodeKind::Text(s) = &mut self.nodes[last].kind
        {
            s.push_str(&text);
            return;
        }
        if self.nodes.len() < MAX_NODES {
            let id = self.nodes.len();
            self.nodes.push(Node {
                parent: Some(parent),
                children: vec![],
                kind: NodeKind::Text(text),
            });
            self.nodes[parent].children.push(id);
        }
    }
}
fn push_serialized(out: &mut String, text: &str) {
    let available = MAX_TEXT.saturating_sub(out.len());
    out.push_str(&text[..floor_boundary(text, text.len().min(available))]);
}
fn escape_serialized(out: &mut String, text: &str, attribute: bool) {
    for c in text.chars() {
        if out.len() >= MAX_TEXT {
            break;
        }
        match c {
            '&' => push_serialized(out, "&amp;"),
            '<' => push_serialized(out, "&lt;"),
            '>' => push_serialized(out, "&gt;"),
            '"' if attribute => push_serialized(out, "&quot;"),
            _ => {
                let mut bytes = [0; 4];
                push_serialized(out, c.encode_utf8(&mut bytes));
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
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
fn is_head(tag: &str) -> bool {
    matches!(
        tag,
        "base"
            | "basefont"
            | "bgsound"
            | "link"
            | "meta"
            | "title"
            | "style"
            | "script"
            | "noscript"
            | "template"
    )
}
fn closes_p(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "div"
            | "dl"
            | "fieldset"
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
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "ul"
    )
}

pub fn parse(source: &str) -> Document {
    let source = &source[..floor_boundary(source, source.len().min(MAX_TEXT))];
    let mut doc = Document {
        nodes: vec![Node {
            parent: None,
            children: vec![],
            kind: NodeKind::Document,
        }],
        root: 0,
        retained_bytes: 0,
    };
    let html = doc.create_element("html");
    doc.append_child(0, html);
    let head = doc.create_element("head");
    doc.append_child(html, head);
    let body = doc.create_element("body");
    doc.append_child(html, body);
    let mut stack = vec![html];
    let mut body_started = false;
    let mut in_head = false;
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() && doc.nodes.len() < MAX_NODES {
        if bytes[i] != b'<' {
            let start = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let text = decode_entities(&source[start..i]);
            if stack.len() == 1 && !body_started && text.trim().is_empty() {
                continue;
            }
            if stack.len() == 1 {
                if in_head && !text.trim().is_empty() {
                    in_head = false;
                }
                let target = if in_head { head } else { body };
                if !in_head {
                    body_started = true;
                }
                doc.push_text(target, text);
            } else {
                doc.push_text(*stack.last().unwrap_or(&body), text);
            }
            continue;
        }
        if source[i..].starts_with("<!--") {
            i = source[i + 4..]
                .find("-->")
                .map(|n| i + 4 + n + 3)
                .unwrap_or(bytes.len());
            continue;
        }
        if source[i..].starts_with("<!") || source[i..].starts_with("<?") {
            i = source[i..]
                .find('>')
                .map(|n| i + n + 1)
                .unwrap_or(bytes.len());
            continue;
        }
        let closing = bytes.get(i + 1) == Some(&b'/');
        let mut p = i + if closing { 2 } else { 1 };
        if !bytes.get(p).is_some_and(u8::is_ascii_alphabetic) {
            let parent = *stack.last().filter(|_| stack.len() > 1).unwrap_or(&body);
            doc.push_text(parent, "<".into());
            i += 1;
            continue;
        }
        let start = p;
        while p < bytes.len() && !is_space(bytes[p]) && !matches!(bytes[p], b'/' | b'>') {
            p += 1;
        }
        let tag = source[start..p].to_ascii_lowercase();
        if closing {
            i = source[p..]
                .find('>')
                .map(|n| p + n + 1)
                .unwrap_or(bytes.len());
            if tag == "head" {
                in_head = false;
                stack.truncate(1);
                continue;
            }
            if matches!(tag.as_str(), "body" | "html") {
                stack.truncate(1);
                body_started = true;
                continue;
            }
            if let Some(index) = stack.iter().rposition(|id| doc.tag(*id) == Some(&tag))
                && index > 0
            {
                stack.truncate(index);
            }
            continue;
        }
        let mut attrs = BTreeMap::new();
        let mut self_closing = false;
        while p < bytes.len() {
            while p < bytes.len() && is_space(bytes[p]) {
                p += 1;
            }
            if p >= bytes.len() {
                break;
            }
            if bytes[p] == b'>' {
                p += 1;
                break;
            }
            if bytes[p] == b'/' {
                self_closing = true;
                p += 1;
                continue;
            }
            let start = p;
            while p < bytes.len() && !is_space(bytes[p]) && !matches!(bytes[p], b'=' | b'>' | b'/')
            {
                p += 1;
            }
            if start == p {
                p += 1;
                continue;
            }
            let name = source[start..p].to_ascii_lowercase();
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
                    let start = p;
                    while p < bytes.len() && bytes[p] != quote {
                        p += 1;
                    }
                    value = decode_entities_context(&source[start..p], true);
                    if p < bytes.len() {
                        p += 1;
                    }
                } else {
                    let start = p;
                    while p < bytes.len() && !is_space(bytes[p]) && bytes[p] != b'>' {
                        p += 1;
                    }
                    value = decode_entities_context(&source[start..p], true);
                }
            }
            if attrs.len() < 1024 {
                attrs.entry(name).or_insert(value);
            }
        }
        i = p;
        if matches!(tag.as_str(), "html" | "head" | "body") {
            let target = match tag.as_str() {
                "html" => html,
                "head" => {
                    in_head = !body_started;
                    head
                }
                _ => {
                    body_started = true;
                    in_head = false;
                    body
                }
            };
            for (k, v) in attrs {
                if doc.attr(target, &k).is_none() {
                    doc.set_attr(target, &k, &v);
                }
            }
            stack.truncate(1);
            continue;
        }
        if closes_p(&tag)
            && let Some(index) = stack.iter().rposition(|id| doc.tag(*id) == Some("p"))
        {
            stack.truncate(index);
        }
        let close_same = match tag.as_str() {
            "li" => Some("li"),
            "dt" | "dd" => Some("dt,dd"),
            "tr" => Some("tr"),
            "td" | "th" => Some("td,th"),
            "option" => Some("option"),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some("h1,h2,h3,h4,h5,h6"),
            "a" => Some("a"),
            "button" => Some("button"),
            _ => None,
        };
        if let Some(tags) = close_same
            && let Some(index) = stack.iter().rposition(|id| {
                doc.tag(*id)
                    .is_some_and(|t| tags.split(',').any(|x| x == t))
            })
            && index > 0
        {
            stack.truncate(index);
        }
        if stack.is_empty() {
            stack.push(html);
        }
        let parent = if stack.len() > 1 {
            *stack.last().unwrap_or(&body)
        } else if !body_started && is_head(&tag) {
            head
        } else {
            body_started = true;
            in_head = false;
            body
        };
        // The table row-group wrapper is implied by HTML's table insertion mode.
        let parent = if tag == "tr" && doc.tag(parent) == Some("table") {
            let rowgroup = doc.create_element("tbody");
            doc.append_child(parent, rowgroup);
            if stack.len() < MAX_DEPTH - 2 {
                stack.push(rowgroup);
            }
            rowgroup
        } else {
            parent
        };
        let id = doc.create_element(&tag);
        if id == doc.root {
            break;
        }
        for (name, value) in attrs {
            doc.set_attr(id, &name, &value);
        }
        doc.append_child(parent, id);
        let in_foreign_content = matches!(tag.as_str(), "svg" | "math")
            || stack
                .iter()
                .rev()
                .find_map(|id| match doc.tag(*id) {
                    Some("svg" | "math") => Some(true),
                    Some("foreignobject") => Some(false),
                    _ => None,
                })
                .unwrap_or(false);
        if matches!(
            tag.as_str(),
            "script" | "style" | "title" | "textarea" | "xmp" | "iframe" | "noembed" | "noframes"
        ) {
            let close = format!("</{tag}");
            let mut end = i;
            while end < bytes.len() {
                if bytes[end] == b'<'
                    && source
                        .get(end..end + close.len())
                        .is_some_and(|s| s.eq_ignore_ascii_case(&close))
                    && bytes
                        .get(end + close.len())
                        .is_none_or(|b| is_space(*b) || *b == b'>')
                {
                    break;
                }
                end += 1;
            }
            let raw = &source[i..end];
            doc.push_text(
                id,
                if matches!(tag.as_str(), "title" | "textarea") {
                    decode_entities(raw)
                } else {
                    raw.to_owned()
                },
            );
            i = if end < bytes.len() {
                source[end..]
                    .find('>')
                    .map(|n| end + n + 1)
                    .unwrap_or(bytes.len())
            } else {
                end
            };
        } else if tag == "plaintext" {
            doc.push_text(id, source[i..].to_owned());
            i = bytes.len();
        } else if !(is_void(&tag) || self_closing && in_foreign_content)
            && stack.len() < MAX_DEPTH - 3
        {
            stack.push(id);
        }
    }
    doc
}

pub fn decode_entities(s: &str) -> String {
    decode_entities_context(s, false)
}
fn decode_entities_context(s: &str, in_attribute: bool) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'&' {
            let c = s[i..].chars().next().unwrap_or('\u{fffd}');
            out.push(if c == '\0' { '\u{fffd}' } else { c });
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
    split_top_level(selector, ',')
        .into_iter()
        .take(128)
        .any(|s| matches_complex(doc, id, s, 0, budget))
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
            if !tag.eq_ignore_ascii_case(&s[..end]) {
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
                } else {
                    if let Some(classes) = doc.attr(id, "class") {
                        if classes.len() > *budget {
                            *budget = 0;
                            return false;
                        }
                        *budget -= classes.len();
                        classes.split_ascii_whitespace().any(|x| x == value)
                    } else {
                        false
                    }
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
                        siblings().into_iter().find(|n| doc.tag(*n) == Some(tag)) == Some(id)
                    }
                    "last-of-type" => {
                        siblings()
                            .into_iter()
                            .rev()
                            .find(|n| doc.tag(*n) == Some(tag))
                            == Some(id)
                    }
                    "only-of-type" => {
                        siblings()
                            .into_iter()
                            .filter(|n| doc.tag(*n) == Some(tag))
                            .count()
                            == 1
                    }
                    "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type" => {
                        let mut all = siblings();
                        if name.contains("of-type") {
                            all.retain(|n| doc.tag(*n) == Some(tag));
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
                        .all(|n| matches!(&doc.nodes[*n].kind,NodeKind::Text(t) if t.is_empty())),
                    "checked" => {
                        doc.attr(id, "checked").is_some() || doc.attr(id, "selected").is_some()
                    }
                    "disabled" => doc.attr(id, "disabled").is_some(),
                    "enabled" => {
                        matches!(
                            tag,
                            "input"
                                | "select"
                                | "textarea"
                                | "button"
                                | "option"
                                | "optgroup"
                                | "fieldset"
                        ) && doc.attr(id, "disabled").is_none()
                    }
                    "required" => doc.attr(id, "required").is_some(),
                    "optional" => {
                        matches!(tag, "input" | "select" | "textarea")
                            && doc.attr(id, "required").is_none()
                    }
                    "link" | "any-link" => {
                        matches!(tag, "a" | "area" | "link") && doc.attr(id, "href").is_some()
                    }
                    "lang" => {
                        let lang = argument.trim_matches(['\'', '"']);
                        let mut n = Some(id);
                        let mut found = false;
                        let mut count = 0;
                        while let Some(p) = n {
                            if let Some(v) = doc.attr(p, "lang") {
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
    let expected = raw.trim_matches(['\'', '"']);
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
        let Some(diff) = (index as i64).checked_sub(b) else {
            return false;
        };
        a == 0 && diff == 0 || a != 0 && diff % a == 0 && diff / a >= 0
    } else {
        s.parse::<usize>().ok() == Some(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialization_escapes_markup_and_preserves_foreign_siblings() {
        let d = parse(
            "<svg viewBox='0 0 20 20'><g/><rect width='10'/><text>&amp;&lt;</text></svg><input value='&quot;&amp;'><script>a < b && c</script>",
        );
        let svg = d.query_selector("svg").unwrap();
        assert_eq!(d.nodes[svg].children.len(), 3);
        assert_eq!(
            d.outer_html(svg),
            "<svg viewbox=\"0 0 20 20\"><g></g><rect width=\"10\"></rect><text>&amp;&lt;</text></svg>"
        );
        assert_eq!(
            d.outer_html(d.query_selector("input").unwrap()),
            "<input value=\"&quot;&amp;\">"
        );
        assert_eq!(
            d.outer_html(d.query_selector("script").unwrap()),
            "<script>a < b && c</script>"
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
        assert!(!nth_matches("n-9223372036854775808", 2));
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
                NodeKind::Text(s) => s.len(),
                NodeKind::Element(e) => {
                    e.tag.len()
                        + e.attrs
                            .iter()
                            .map(|(k, v)| k.len() + v.len())
                            .sum::<usize>()
                }
                NodeKind::Document => 0,
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
}
