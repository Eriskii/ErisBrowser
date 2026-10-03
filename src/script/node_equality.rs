//! Exact equality of the represented ordinary DOM tree. Hosted template
//! contents and mutable JavaScript properties do not supply comparison fields.
//! Reached graph checks are local; an early mismatch is not a whole-tree audit.
use super::*;
use crate::dom::{AttributeNamespace, DomString, Element, Node};
use std::cmp::Ordering;
use std::collections::btree_map::{Entry, Iter};
use std::mem::size_of;

#[cfg(test)]
mod tests;

pub(super) const METADATA_OBJECTS: usize = 1;
const PAGE_BITS: usize = 64;
const MAX_PAGES: usize = crate::dom::MAX_NODES.div_ceil(PAGE_BITS);

fn invalid() -> ScriptError {
    ScriptError::type_error("invalid Node equality tree")
}
fn overflow() -> ScriptError {
    ScriptError::resource("Node equality work or storage overflow")
}
fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or_else(overflow)
}
fn mul(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or_else(overflow)
}
// Callers prepay setup and bound count before this conservative height walk.
fn height(count: usize) -> Result<usize> {
    if count == 0 {
        return Ok(0);
    }
    let mut capacity = add(count, 1)? / 2;
    let mut height = 1;
    while capacity >= 6 {
        capacity /= 6;
        height += 1;
    }
    Ok(height)
}

#[derive(Clone, Copy)]
struct EqualityFrame {
    left: NodeId,
    right: NodeId,
    next: usize,
}

struct Attributes<'a> {
    values: Iter<'a, String, String>,
    namespaces: Iter<'a, String, AttributeNamespace>,
    namespace: Option<(&'a String, &'a AttributeNamespace)>,
    value_work: usize,
    namespace_work: usize,
}
type AttributeRow<'a> = (&'a String, &'a String, Option<AttributeNamespace>);

impl<'a> Attributes<'a> {
    fn new(runtime: &mut Runtime, element: &'a Element) -> Result<Self> {
        let value_work = add(8, mul(8, height(element.attrs.len())?)?)?;
        let namespace_work = add(8, mul(8, height(element.attr_namespaces.len())?)?)?;
        runtime.work(value_work)?;
        let values = element.attrs.iter();
        runtime.work(namespace_work)?;
        let mut namespaces = element.attr_namespaces.iter();
        runtime.work(namespace_work)?;
        let namespace = namespaces.next();
        Ok(Self {
            values,
            namespaces,
            namespace,
            value_work,
            namespace_work,
        })
    }

    fn next(&mut self, runtime: &mut Runtime) -> Result<Option<AttributeRow<'a>>> {
        runtime.work(8)?;
        runtime.work(self.value_work)?;
        let Some((key, value)) = self.values.next() else {
            return if self.namespace.is_some() {
                Err(invalid())
            } else {
                Ok(None)
            };
        };
        let namespace = if let Some((name, namespace)) = self.namespace {
            match runtime.equality_key_order(name.as_bytes(), key.as_bytes())? {
                Ordering::Less => return Err(invalid()),
                Ordering::Greater => None,
                Ordering::Equal => {
                    // Eleven fixed ASCII names, at most thirteen bytes each.
                    runtime.work(8 + 11 * (1 + 13))?;
                    if AttributeNamespace::from_qualified_name(name) != Some(*namespace) {
                        return Err(invalid());
                    }
                    runtime.work(self.namespace_work)?;
                    self.namespace = self.namespaces.next();
                    Some(*namespace)
                }
            }
        } else {
            None
        };
        Ok(Some((key, value, namespace)))
    }
}

impl Runtime {
    pub(super) fn install_node_equality(&mut self, prototype: usize) -> Result<()> {
        self.work(4)?;
        let method = self.node_data_function("isEqualNode", "isEqualNode", 1)?;
        self.dom_proto_named(
            prototype,
            "isEqualNode",
            Property::data(method, true, true, true),
        )
    }

    fn equality_local<'a>(&mut self, doc: &'a Document, id: NodeId) -> Result<&'a Node> {
        self.work(16)?;
        let node = node_predicates::local_node(doc, id)?;
        if id == doc.root && !matches!(node.kind, NodeKind::Document) {
            return Err(invalid());
        }
        if let NodeKind::Element(element) = &node.kind
            && (element.attrs.len() > 1024
                || element.attr_namespaces.len() > element.attrs.len()
                || element.attr_namespaces.len() > 11)
        {
            return Err(invalid());
        }
        Ok(node)
    }

    fn equality_slice<T: PartialEq>(&mut self, left: &[T], right: &[T]) -> Result<bool> {
        self.work(8)?;
        if left.len() != right.len() {
            return Ok(false);
        }
        self.work(add(1, mul(4, left.len())?)?)?;
        Ok(left == right)
    }

    fn equality_key_order(&mut self, left: &[u8], right: &[u8]) -> Result<Ordering> {
        self.work(8)?;
        self.work(add(1, mul(4, left.len().min(right.len()))?)?)?;
        Ok(left.cmp(right))
    }

    fn equality_data(&mut self, left: &DomString, right: &DomString) -> Result<bool> {
        self.work(8)?;
        match (left.scalar(), right.scalar()) {
            (Some(left), Some(right)) => self.equality_slice(left.as_bytes(), right.as_bytes()),
            (None, None) => self.equality_slice(
                left.raw_units().ok_or_else(invalid)?,
                right.raw_units().ok_or_else(invalid)?,
            ),
            // Canonical Units contains an unmatched unit; canonical Scalar
            // cannot. No decoding, temporary payload, or JS string cap applies.
            _ => Ok(false),
        }
    }

    fn equality_attributes(&mut self, left: &Element, right: &Element) -> Result<bool> {
        self.work(32)?;
        if left.attrs.len() != right.attrs.len()
            || left.attr_namespaces.len() != right.attr_namespaces.len()
        {
            return Ok(false);
        }
        if left.attrs.is_empty() && left.attr_namespaces.is_empty() {
            return Ok(true);
        }
        let mut left = Attributes::new(self, left)?;
        let mut right = Attributes::new(self, right)?;
        loop {
            self.work(8)?;
            match (left.next(self)?, right.next(self)?) {
                (None, None) => return Ok(true),
                (Some((ak, av, an)), Some((bk, bv, bn))) => {
                    if !self.equality_slice(ak.as_bytes(), bk.as_bytes())? {
                        return Ok(false);
                    }
                    self.work(4)?;
                    if an != bn || !self.equality_slice(av.as_bytes(), bv.as_bytes())? {
                        return Ok(false);
                    }
                }
                _ => return Err(invalid()),
            }
        }
    }

    fn equality_fields(&mut self, left: &Node, right: &Node) -> Result<bool> {
        self.work(16)?;
        if left.children.len() != right.children.len() {
            return Ok(false);
        }
        match (&left.kind, &right.kind) {
            (NodeKind::Document, NodeKind::Document)
            | (NodeKind::DocumentFragment { .. }, NodeKind::DocumentFragment { .. }) => Ok(true),
            (NodeKind::Text(a), NodeKind::Text(b))
            | (NodeKind::Comment(a), NodeKind::Comment(b)) => self.equality_data(a, b),
            (
                NodeKind::ProcessingInstruction {
                    target: at,
                    data: ad,
                },
                NodeKind::ProcessingInstruction {
                    target: bt,
                    data: bd,
                },
            ) => {
                Ok(self.equality_slice(at.as_bytes(), bt.as_bytes())?
                    && self.equality_data(ad, bd)?)
            }
            (NodeKind::Doctype(a), NodeKind::Doctype(b)) => Ok(self
                .equality_slice(a.name.as_bytes(), b.name.as_bytes())?
                && self.equality_slice(
                    a.public_id.as_deref().unwrap_or("").as_bytes(),
                    b.public_id.as_deref().unwrap_or("").as_bytes(),
                )?
                && self.equality_slice(
                    a.system_id.as_deref().unwrap_or("").as_bytes(),
                    b.system_id.as_deref().unwrap_or("").as_bytes(),
                )?),
            (NodeKind::Element(a), NodeKind::Element(b)) => Ok(a.namespace == b.namespace
                && self.equality_slice(a.tag.as_bytes(), b.tag.as_bytes())?
                && self.equality_attributes(a, b)?),
            _ => Ok(false),
        }
    }

    fn equality_mark(&mut self, pages: &mut BTreeMap<usize, u64>, id: NodeId) -> Result<()> {
        self.work(16)?;
        let count = pages.len();
        if id >= crate::dom::MAX_NODES || count > MAX_PAGES {
            return Err(invalid());
        }
        let page = id / PAGE_BITS;
        let bit = u32::try_from(id % PAGE_BITS).map_err(|_| overflow())?;
        let mask = 1_u64.checked_shl(bit).ok_or_else(overflow)?;
        if page >= MAX_PAGES {
            return Err(invalid());
        }
        let height = height(count)?;
        let levels = add(height, 1)?;
        let comparisons = count.min(mul(11, height)?);
        self.work(add(comparisons, add(mul(8, levels)?, 4)?)?)?;
        // Pinned Entry does not allocate for an absent root. Only the vacant
        // branch performs growth, after its separate work and typed admission.
        match pages.entry(page) {
            Entry::Occupied(mut entry) => {
                self.work(8)?;
                let old = *entry.get();
                if old & mask != 0 {
                    return Err(invalid());
                }
                *entry.get_mut() = old | mask;
            }
            Entry::Vacant(entry) => {
                if count == MAX_PAGES {
                    return Err(invalid());
                }
                // Includes both internal split-side and insertion-suffix
                // backlink repairs, movement, and eventual consuming teardown.
                self.work(add(mul(256, levels)?, 32)?)?;
                let blocks = if count == 0 {
                    1
                } else if count < 11 {
                    0
                } else {
                    levels
                };
                let bytes = add(
                    mul(16, add(size_of::<usize>(), size_of::<u64>())?)?,
                    add(mul(32, size_of::<usize>())?, 64)?,
                )?;
                self.charge(mul(blocks, bytes)?)?;
                entry.insert(mask);
            }
        }
        Ok(())
    }

    fn equality_push(
        &mut self,
        frames: &mut Vec<EqualityFrame>,
        frame: EqualityFrame,
    ) -> Result<()> {
        self.work(5)?; // push plus eventual pop and buffer cleanup
        let maximum = add(crate::dom::MAX_DEPTH, 1)?;
        if frames.len() >= maximum {
            return Err(invalid());
        }
        if frames.len() == frames.capacity() {
            let capacity = add(frames.len(), 1)?.max(mul(2, frames.capacity())?.min(maximum));
            self.work(add(1, mul(6, frames.len())?)?)?;
            self.charge(mul(capacity, size_of::<EqualityFrame>())?)?;
            frames
                .try_reserve_exact(capacity - frames.len())
                .map_err(|_| ScriptError::resource("Node equality cursor allocation failed"))?;
        }
        frames.push(frame);
        Ok(())
    }

    pub(super) fn node_is_equal_node(
        &mut self,
        receiver: Value,
        args: &[Value],
        doc: &Document,
    ) -> Result<Value> {
        self.work(16)?;
        let left = node_predicates::node_id(&receiver, doc)?;
        let Some(right) = self.predicate_argument(args, doc)? else {
            self.work(2)?;
            return Ok(Value::Bool(false));
        };
        let a = self.equality_local(doc, left)?;
        let b = self.equality_local(doc, right)?;
        self.work(2)?;
        if left == right {
            return Ok(Value::Bool(true));
        }
        if !self.equality_fields(a, b)? {
            return Ok(Value::Bool(false));
        }
        if a.children.is_empty() {
            return Ok(Value::Bool(true));
        }
        let mut left_pages = BTreeMap::new();
        let mut right_pages = BTreeMap::new();
        let mut frames = Vec::new();
        self.equality_mark(&mut left_pages, left)?;
        self.equality_mark(&mut right_pages, right)?;
        self.equality_push(
            &mut frames,
            EqualityFrame {
                left,
                right,
                next: 0,
            },
        )?;
        loop {
            self.work(8)?;
            let index = frames.len() - 1;
            let EqualityFrame { left, right, next } = frames[index];
            if next == doc.nodes[left].children.len() {
                frames.pop();
                if frames.is_empty() {
                    return Ok(Value::Bool(true));
                }
                continue;
            }
            self.work(16)?;
            if frames.len() > crate::dom::MAX_DEPTH {
                return Err(invalid());
            }
            let a = doc.nodes[left].children[next];
            let b = doc.nodes[right].children[next];
            frames[index].next = add(next, 1)?;
            let an = doc.nodes.get(a).ok_or_else(invalid)?;
            let bn = doc.nodes.get(b).ok_or_else(invalid)?;
            for (id, node, parent) in [(a, an, left), (b, bn, right)] {
                if node.parent != Some(parent)
                    || id == doc.root
                    || matches!(
                        node.kind,
                        NodeKind::Document | NodeKind::DocumentFragment { .. }
                    )
                    || (matches!(node.kind, NodeKind::Doctype(_)) && parent != doc.root)
                {
                    return Err(invalid());
                }
            }
            let an = self.equality_local(doc, a)?;
            let bn = self.equality_local(doc, b)?;
            self.equality_mark(&mut left_pages, a)?;
            self.equality_mark(&mut right_pages, b)?;
            if !self.equality_fields(an, bn)? {
                return Ok(Value::Bool(false));
            }
            if !an.children.is_empty() {
                self.equality_push(
                    &mut frames,
                    EqualityFrame {
                        left: a,
                        right: b,
                        next: 0,
                    },
                )?;
            }
        }
    }
}
