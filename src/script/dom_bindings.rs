//! Checked conversion and receiver dispatch for the supported DOM operations.
use super::*;
use std::collections::btree_map::Entry;

mod append;

#[cfg(test)]
mod append_tests;
#[cfg(test)]
mod identity_tests;

// Unrelated operations keep their original order, installation and access fees.
const METHODS: &[(&str, usize)] = &[
    ("DOM.getElementById", 1),
    ("DOM.getElementsByTagName", 1),
    ("DOM.getElementsByClassName", 1),
    ("DOM.createElement", 1),
    ("DOM.createTextNode", 1),
    ("DOM.createDocumentFragment", 0),
    ("DOM.getAttribute", 1),
    ("DOM.hasAttribute", 1),
    ("DOM.setAttribute", 2),
    ("DOM.removeAttribute", 1),
    ("DOM.appendChild", 1),
    ("DOM.removeChild", 1),
    ("DOM.remove", 0),
    ("DOM.cloneNode", 0),
    ("DOMTokenList.add", 0),
    ("DOMTokenList.remove", 0),
    ("DOMTokenList.toggle", 1),
    ("DOMTokenList.contains", 1),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParentInterface {
    Document,
    Element,
    DocumentFragment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ParentOperation {
    QuerySelector,
    QuerySelectorAll,
    Append,
}

#[derive(Debug)]
struct ParentMethod {
    full: &'static str,
    suffix: &'static str,
    name: &'static str,
    length: usize,
    interface: ParentInterface,
    operation: ParentOperation,
}

static PARENT_METHODS: [ParentMethod; 9] = [
    ParentMethod {
        full: "DOM.Document.querySelector",
        suffix: "Document.querySelector",
        name: "querySelector",
        length: 1,
        interface: ParentInterface::Document,
        operation: ParentOperation::QuerySelector,
    },
    ParentMethod {
        full: "DOM.Element.querySelector",
        suffix: "Element.querySelector",
        name: "querySelector",
        length: 1,
        interface: ParentInterface::Element,
        operation: ParentOperation::QuerySelector,
    },
    ParentMethod {
        full: "DOM.DocumentFragment.querySelector",
        suffix: "DocumentFragment.querySelector",
        name: "querySelector",
        length: 1,
        interface: ParentInterface::DocumentFragment,
        operation: ParentOperation::QuerySelector,
    },
    ParentMethod {
        full: "DOM.Document.querySelectorAll",
        suffix: "Document.querySelectorAll",
        name: "querySelectorAll",
        length: 1,
        interface: ParentInterface::Document,
        operation: ParentOperation::QuerySelectorAll,
    },
    ParentMethod {
        full: "DOM.Element.querySelectorAll",
        suffix: "Element.querySelectorAll",
        name: "querySelectorAll",
        length: 1,
        interface: ParentInterface::Element,
        operation: ParentOperation::QuerySelectorAll,
    },
    ParentMethod {
        full: "DOM.DocumentFragment.querySelectorAll",
        suffix: "DocumentFragment.querySelectorAll",
        name: "querySelectorAll",
        length: 1,
        interface: ParentInterface::DocumentFragment,
        operation: ParentOperation::QuerySelectorAll,
    },
    ParentMethod {
        full: "DOM.Element.append",
        suffix: "Element.append",
        name: "append",
        length: 0,
        interface: ParentInterface::Element,
        operation: ParentOperation::Append,
    },
    ParentMethod {
        full: "DOM.DocumentFragment.append",
        suffix: "DocumentFragment.append",
        name: "append",
        length: 0,
        interface: ParentInterface::DocumentFragment,
        operation: ParentOperation::Append,
    },
    ParentMethod {
        full: "DOM.Document.append",
        suffix: "Document.append",
        name: "append",
        length: 0,
        interface: ParentInterface::Document,
        operation: ParentOperation::Append,
    },
];

#[cfg(test)]
fn parent_method(
    interface: ParentInterface,
    operation: ParentOperation,
) -> Option<&'static ParentMethod> {
    use ParentInterface::{Document, DocumentFragment, Element};
    use ParentOperation::{Append, QuerySelector, QuerySelectorAll};
    let index = match (interface, operation) {
        (Document, QuerySelector) => 0,
        (Element, QuerySelector) => 1,
        (DocumentFragment, QuerySelector) => 2,
        (Document, QuerySelectorAll) => 3,
        (Element, QuerySelectorAll) => 4,
        (DocumentFragment, QuerySelectorAll) => 5,
        (Element, Append) => 6,
        (DocumentFragment, Append) => 7,
        (Document, Append) => 8,
    };
    Some(&PARENT_METHODS[index])
}

fn parent_interface(receiver: &Value, doc: &Document) -> Option<ParentInterface> {
    let id = match receiver {
        Value::Document => doc.root,
        Value::Node(id) => *id,
        _ => return None,
    };
    match &doc.nodes.get(id)?.kind {
        NodeKind::Document => Some(ParentInterface::Document),
        NodeKind::Element(_) => Some(ParentInterface::Element),
        NodeKind::DocumentFragment { .. } => Some(ParentInterface::DocumentFragment),
        _ => None,
    }
}

// Only the fixed five-byte namespace guard precedes the scoped paid resolver.
pub(super) fn is_parent_method_name(name: &str) -> bool {
    name.as_bytes().get(..4) == Some(b"DOM.") && matches!(name.as_bytes().get(4), Some(b'D' | b'E'))
}

// A bootstrap reservation, not a runtime object limit. The current complete
// intrinsic inventory fits this existing capacity; private checks bind that fact.
const BOOTSTRAP_OBJECT_CAPACITY: usize = dom_prototypes::BOOTSTRAP_OBJECTS;
const PARENT_INSTALL_ORDER: [usize; 9] = [8, 0, 3, 7, 2, 5, 6, 1, 4];
const PARENT_SHARED_WORK: usize = 8 + 90 + 4;

fn parent_node_bytes<K, V>() -> usize {
    16 * (std::mem::size_of::<K>() + std::mem::size_of::<V>())
        + 32 * std::mem::size_of::<usize>()
        + 64
}

fn parent_install_work(ordinal: usize, row: &ParentMethod) -> usize {
    // The guarded sorted batch fits one B=6 leaf (capacity eleven). Entry
    // searches each earlier key once; right-edge insertion moves no old entry.
    // Eight logical entry/length/handle units are not byte-copy counts. The
    // first root's setup is covered by PARENT_SHARED_WORK.
    (64 + row.full.len() + 8 + 16).saturating_add(ordinal.saturating_mul(1 + row.full.len() / 8))
}

fn parent_install_bytes(row: &ParentMethod) -> usize {
    // Retain the original literal payload allowance, ordinary object base and
    // both ordinary property debits, despite direct final-descriptor creation.
    1536 + 8 * (row.full.len() + row.name.len())
        + parent_node_bytes::<PropertyKey, Property>()
        + 4 * std::mem::size_of::<PropertyKey>()
        + 72
        + std::mem::size_of::<Option<AbortSlot>>()
        + std::mem::size_of::<Option<f64>>()
        + 552
}

fn parent_ascii(text: &str) -> Result<JsString> {
    // Only the five static ASCII literals reach this builder, after admission.
    // Reserving first avoids geometric Vec relocation in the two-pass debit.
    let mut units = Vec::new();
    units
        .try_reserve_exact(text.len())
        .map_err(|_| ScriptError::resource("DOM method string allocation failed"))?;
    units.extend(text.bytes().map(u16::from));
    Ok(units.into())
}

fn ascii_space(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{c}' | '\r' | ' ')
}

enum AppendValue {
    Node(NodeId),
    Text(JsString),
}

impl Runtime {
    pub(super) fn install_dom_parent_prototype(&mut self, name: &str, owner: usize) -> Result<()> {
        self.work(3 * PARENT_METHODS.len())?;
        for row in &PARENT_METHODS {
            let interface = match row.interface {
                ParentInterface::Document => "Document",
                ParentInterface::Element => "Element",
                ParentInterface::DocumentFragment => "DocumentFragment",
            };
            if interface != name {
                continue;
            }
            self.work(8 + row.full.len())?;
            self.charge(std::mem::size_of::<Native>() + 32 + row.full.len())?;
            let function = Self::native(row.full, Value::Undefined);
            self.dom_proto_named(owner, row.name, Property::data(function, true, true, true))?;
        }
        Ok(())
    }

    pub(super) fn initialize_dom_bindings(&mut self) -> Result<()> {
        for &(full, length) in METHODS {
            self.work(full.len() + 1)?;
            self.charge(512 + full.len() * 8)?;
            self.intrinsic_function(full, full.rsplit('.').next().unwrap(), length)?;
        }
        Ok(())
    }

    pub(super) fn reserve_bootstrap_objects(&mut self) -> Result<()> {
        if self.objects.capacity() >= BOOTSTRAP_OBJECT_CAPACITY {
            return Ok(());
        }
        let length = self.objects.len();
        let additional = BOOTSTRAP_OBJECT_CAPACITY
            .checked_sub(length)
            .ok_or_else(|| ScriptError::resource("bootstrap object arena overflow"))?;
        let bytes = BOOTSTRAP_OBJECT_CAPACITY
            .checked_mul(std::mem::size_of::<ScriptObject>())
            .ok_or_else(|| ScriptError::resource("bootstrap object arena overflow"))?;
        self.work(1usize.saturating_add(length.saturating_mul(2)))?;
        // Full requested block, with no refund for any previous allocation.
        // Real bootstrap reaches this with zero objects, before machine setup.
        self.charge(bytes)?;
        self.objects
            .try_reserve_exact(additional)
            .map_err(|_| ScriptError::resource("bootstrap object arena allocation failed"))
    }

    pub(super) fn initialize_dom_parent_bindings(&mut self) -> Result<()> {
        self.work(8)?;
        if !self.native_properties.is_empty()
            || self.functions.get(self.function_prototype).is_none()
        {
            return Err(ScriptError::resource(
                "DOM method bootstrap state is not fresh",
            ));
        }
        // Five immutable UTF-16 strings: 45 units encoded then copied once.
        // Four further units initialize the one fresh registry leaf.
        self.work(PARENT_SHARED_WORK - 8)?;
        self.charge(512 + parent_node_bytes::<String, usize>())?;
        let keys = [parent_ascii("name")?.into(), parent_ascii("length")?.into()];
        let display = [
            parent_ascii("querySelector")?,
            parent_ascii("querySelectorAll")?,
            parent_ascii("append")?,
        ];
        for (ordinal, index) in PARENT_INSTALL_ORDER.into_iter().enumerate() {
            let row = &PARENT_METHODS[index];
            let display = &display[match row.operation {
                ParentOperation::QuerySelector => 0,
                ParentOperation::QuerySelectorAll => 1,
                ParentOperation::Append => 2,
            }];
            self.install_dom_parent_method(ordinal, row, display, &keys)?;
        }
        Ok(())
    }

    fn install_dom_parent_method(
        &mut self,
        ordinal: usize,
        row: &ParentMethod,
        display: &JsString,
        keys: &[PropertyKey; 2],
    ) -> Result<()> {
        self.work(parent_install_work(ordinal, row))?;
        if ordinal >= PARENT_INSTALL_ORDER.len()
            || self.native_properties.len() != ordinal
            || !std::ptr::eq(row, &PARENT_METHODS[PARENT_INSTALL_ORDER[ordinal]])
            || self.objects.len() == self.objects.capacity()
        {
            return Err(ScriptError::resource("DOM method batch invariant violated"));
        }
        self.charge(parent_install_bytes(row))?;
        let mut bag = ScriptObject {
            prototype: Some(Value::Function(self.function_prototype)),
            ..ScriptObject::default()
        };
        bag.order
            .try_reserve_exact(4)
            .map_err(|_| ScriptError::resource("DOM method order allocation failed"))?;
        // One fresh-leaf comparison, three entry moves, four setup operations,
        // two order appends and six final descriptor flags: sixteen units.
        for (key, value) in keys.iter().zip([
            Value::String(display.clone()),
            Value::Number(row.length as f64),
        ]) {
            let Entry::Vacant(entry) = bag.values.entry(key.clone()) else {
                unreachable!()
            };
            bag.order.push(key.clone());
            entry.insert(Property::data(value, false, false, true));
        }
        let id = self.objects.len();
        let Entry::Vacant(entry) = self.native_properties.entry(row.full.to_owned()) else {
            return Err(ScriptError::resource("duplicate DOM method metadata"));
        };
        // Every admission/allocation except the prepaid registry leaf precedes
        // publication. No callback or whole-Runtime call crosses this borrow.
        self.objects.push(bag);
        entry.insert(id);
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn dom_parent_method(
        &mut self,
        receiver: &Value,
        operation: ParentOperation,
        doc: &Document,
    ) -> Result<Value> {
        self.work(4)?;
        let Some(interface) = parent_interface(receiver, doc) else {
            return Ok(Value::Undefined);
        };
        self.work(2)?;
        let Some(row) = parent_method(interface, operation) else {
            return Ok(Value::Undefined);
        };
        self.work(8 + row.full.len())?;
        self.charge(
            32 + std::mem::size_of::<Native>() + 2 * std::mem::size_of::<usize>() + row.full.len(),
        )?;
        Ok(Self::native(row.full, Value::Undefined))
    }

    fn dom_parent_resolve(&mut self, name: &str, full: bool) -> Result<&'static ParentMethod> {
        for row in &PARENT_METHODS {
            let candidate = if full { row.full } else { row.suffix };
            self.tick()?;
            if name.len() == candidate.len() {
                self.work(1 + name.len() / 8)?;
                if name == candidate {
                    return Ok(row);
                }
            }
        }
        Err(ScriptError::type_error("unknown DOM parent method"))
    }

    pub(super) fn dom_parent_call_preflight(&mut self, name: &str) -> Result<()> {
        self.work(4)?;
        let row = self.dom_parent_resolve(name, true)?;
        self.work(4 + row.full.len())?;
        self.charge(32 + row.full.len())
    }

    pub(super) fn dom_method(&mut self, name: &str, tokens: bool) -> Result<Value> {
        let prefix = if tokens { "DOMTokenList." } else { "DOM." };
        let full = METHODS
            .iter()
            .map(|(full, _)| *full)
            .find(|full| full.strip_prefix(prefix) == Some(name))
            .expect("known DOM method");
        self.work(1 + METHODS.len() / 8 + full.len() / 8)?;
        self.charge(128 + full.len())?;
        self.alloc_native(full, Value::Undefined)
    }

    // Legacy scalar-only host interfaces retain this explicit replacement
    // boundary after checked conversion. Exact character-data producers use
    // the canonical DOM data builder instead.
    pub(super) fn dom_string(&mut self, value: Value, doc: &mut Document) -> Result<String> {
        let text = self.string_hint(value, doc)?;
        self.work(1 + text.len() / 8)?;
        self.charge(32 + text.len().saturating_mul(3))?;
        let mut result = String::new();
        result
            .try_reserve_exact(text.len().saturating_mul(3))
            .map_err(|_| ScriptError::resource("DOM string allocation failed"))?;
        for c in char::decode_utf16(text.units().iter().copied()) {
            result.push(c.unwrap_or(char::REPLACEMENT_CHARACTER));
        }
        Ok(result)
    }

    fn dom_throw(&mut self, name: &str, message: &str) -> Result<ScriptError> {
        let value = self.dom_exception(name.into(), message.into())?;
        self.thrown_error(value)
    }

    pub(super) fn dom_native(
        &mut self,
        name: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let name = if matches!(name.as_bytes().first(), Some(b'D' | b'E')) {
            let row = self.dom_parent_resolve(name, false)?;
            self.work(4)?;
            if parent_interface(&receiver, doc) != Some(row.interface) {
                return Err(ScriptError::type_error("incompatible DOM method receiver"));
            }
            // Only normalize after the defining-interface brand succeeds.
            match row.operation {
                ParentOperation::QuerySelector => "querySelector",
                ParentOperation::QuerySelectorAll => "querySelectorAll",
                ParentOperation::Append => "append",
            }
        } else {
            name
        };
        let node = match receiver {
            Value::Document => Some(doc.root),
            Value::Node(id) if id < doc.nodes.len() => Some(id),
            _ => None,
        };
        let parent = node.is_some_and(|id| {
            matches!(
                doc.nodes[id].kind,
                NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_)
            )
        });
        let element = node.is_some_and(|id| matches!(doc.nodes[id].kind, NodeKind::Element(_)));
        let valid = match name {
            "createElement" | "createTextNode" | "createDocumentFragment" => {
                receiver == Value::Document
            }
            "getElementById" => node.is_some_and(|id| {
                matches!(
                    doc.nodes[id].kind,
                    NodeKind::Document | NodeKind::DocumentFragment { .. }
                )
            }),
            "getElementsByTagName" | "getElementsByClassName" => {
                receiver == Value::Document || element
            }
            "querySelector" | "querySelectorAll" | "append" => parent,
            "getAttribute" | "hasAttribute" | "setAttribute" | "removeAttribute" => element,
            "appendChild" | "removeChild" | "cloneNode" => node.is_some(),
            "remove" => node.is_some_and(|id| {
                !matches!(
                    doc.nodes[id].kind,
                    NodeKind::Document | NodeKind::DocumentFragment { .. }
                )
            }),
            _ => false,
        };
        if !valid {
            return Err(ScriptError::type_error("incompatible DOM method receiver"));
        }
        let required = if name == "setAttribute" {
            2
        } else if matches!(
            name,
            "append" | "remove" | "cloneNode" | "createDocumentFragment"
        ) {
            0
        } else {
            1
        };
        if args.len() < required {
            return Err(ScriptError::type_error("missing required DOM argument"));
        }
        let arg = |index: usize| args.get(index).cloned().unwrap_or(Value::Undefined);
        let id = node.unwrap();
        match name {
            "createElement" => {
                let tag = self.dom_string(arg(0), doc)?;
                // The existing element-name subset remains explicit; full XML
                // names, customized built-ins and options are separate work.
                if tag.is_empty()
                    || !tag
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    return Err(
                        self.dom_throw("InvalidCharacterError", "invalid element tag name")?
                    );
                }
                self.ensure_dom_capacity(
                    doc,
                    if tag.eq_ignore_ascii_case("template") {
                        2
                    } else {
                        1
                    },
                )?;
                self.charge(tag.len().saturating_mul(2) + 32)?;
                Ok(Value::Node(doc.create_element(&tag.to_ascii_lowercase())))
            }
            "createTextNode" => {
                let text = self.string_hint(arg(0), doc)?;
                let plan =
                    self.plan_dom_data(text.units().iter().copied(), text.len(), text.len())?;
                self.work(8)?;
                if !doc.admits_text_node(plan.stored_bytes()) {
                    return Err(ScriptError::resource(
                        "DOM text node or data limit exceeded",
                    ));
                }
                self.ensure_dom_capacity(doc, 1)?;
                let data = self.emit_dom_data(plan)?;
                self.work(8)?;
                self.dom_reserve_node_growth(doc, 1)?;
                let id = doc
                    .create_text_node_owned(data)
                    .map_err(processing_instruction::dom_data_error)?;
                Ok(Value::Node(id))
            }
            "createDocumentFragment" => {
                self.ensure_dom_capacity(doc, 1)?;
                Ok(Value::Node(doc.create_document_fragment()))
            }
            "getAttribute" | "hasAttribute" | "removeAttribute" | "setAttribute" => {
                let key = self.dom_string(arg(0), doc)?;
                if name == "setAttribute" {
                    let text = self.dom_string(arg(1), doc)?;
                    self.charge(key.len() + text.len() + 64)?;
                    let work = doc.base_attribute_work(id, &key);
                    self.work(work.saturating_add(if work > 0 { text.len() } else { 0 }))?;
                    self.charge_details_attribute(id, &key, text.len(), doc)?;
                    doc.set_attr(id, &key, &text);
                    self.event_attribute_changed(id, &key, doc)?;
                    return Ok(Value::Undefined);
                }
                if name == "removeAttribute" {
                    self.work(doc.base_attribute_work(id, &key))?;
                    self.charge_details_attribute(id, &key, 0, doc)?;
                    doc.remove_attr(id, &key);
                    self.event_attribute_changed(id, &key, doc)?;
                    return Ok(Value::Undefined);
                }
                let found = doc.attr(id, &key);
                if name == "hasAttribute" {
                    Ok(Value::Bool(found.is_some()))
                } else {
                    match found {
                        Some(value) => self.string(value),
                        None => Ok(Value::Null),
                    }
                }
            }
            "appendChild" | "removeChild" => {
                let child = match arg(0) {
                    Value::Node(child) if child < doc.nodes.len() => child,
                    Value::Document => doc.root,
                    _ => return Err(ScriptError::type_error("expected DOM node")),
                };
                if name == "appendChild" {
                    self.dom_checked_append(id, child, doc)?;
                } else {
                    if doc.nodes[child].parent != Some(id) {
                        return Err(self.dom_throw("NotFoundError", "node is not a child")?);
                    }
                    self.charge_dom_remove(id, doc)?;
                    self.work(doc.base_remove_work(child))?;
                    doc.remove_child(id, child);
                }
                Ok(Value::Node(child))
            }
            "append" => {
                self.dom_append(id, args, doc)?;
                Ok(Value::Undefined)
            }
            "cloneNode" => self
                .clone_dom_node(id, arg(0).truthy(), doc)
                .map(Value::Node),
            "remove" => {
                if let Some(parent) = doc.nodes[id].parent {
                    self.charge_dom_remove(parent, doc)?;
                    self.work(doc.base_remove_work(id))?;
                    doc.remove_child(parent, id);
                }
                Ok(Value::Undefined)
            }
            "querySelector"
            | "querySelectorAll"
            | "getElementById"
            | "getElementsByTagName"
            | "getElementsByClassName" => {
                let input = self.dom_string(arg(0), doc)?;
                self.work(1 + doc.nodes.len() / 8 + input.len() / 8)?;
                self.charge(
                    64 + input.len().saturating_mul(4) + doc.nodes.len().saturating_mul(16),
                )?;
                let selector = if name == "getElementsByClassName" {
                    // Write directly into the charged string: a temporary
                    // Vec<&str> would need much more storage for tiny tokens.
                    let mut selector = String::new();
                    selector
                        .try_reserve_exact(input.len().saturating_add(1))
                        .map_err(|_| ScriptError::resource("class query allocation failed"))?;
                    selector.push('.');
                    for token in input.split(ascii_space).filter(|s| !s.is_empty()) {
                        if selector.len() > 1 {
                            selector.push('.');
                        }
                        selector.push_str(token);
                    }
                    selector
                } else {
                    input.clone()
                };
                let candidates = if name == "getElementById" {
                    doc.query_selector_all_from(id, "*")
                        .into_iter()
                        .filter(|node| doc.attr(*node, "id") == Some(input.as_str()))
                        .collect::<Vec<_>>()
                } else {
                    doc.query_selector_all_from(id, &selector)
                };
                self.charge(
                    candidates
                        .len()
                        .saturating_mul(std::mem::size_of::<Value>()),
                )?;
                if matches!(name, "querySelector" | "getElementById") {
                    Ok(candidates
                        .first()
                        .copied()
                        .map(Value::Node)
                        .unwrap_or(Value::Null))
                } else {
                    self.array(candidates.into_iter().map(Value::Node).collect())
                }
            }
            _ => unreachable!("validated DOM operation"),
        }
    }

    fn class_tokens(&mut self, id: NodeId, doc: &Document) -> Result<Vec<String>> {
        let text = doc.attr(id, "class").unwrap_or("");
        self.work(1 + text.len())?;
        let count = text.split(ascii_space).filter(|s| !s.is_empty()).count();
        self.charge(
            64 + text.len().saturating_mul(2)
                + count.saturating_mul(std::mem::size_of::<String>() + 64),
        )?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("class token allocation failed"))?;
        let mut seen = BTreeSet::new();
        for token in text.split(ascii_space).filter(|s| !s.is_empty()) {
            self.work(
                1 + token
                    .len()
                    .saturating_mul(6)
                    .saturating_mul(1 + count.checked_ilog2().unwrap_or(0) as usize),
            )?;
            if seen.insert(token) {
                result.push(token.to_owned());
            }
        }
        Ok(result)
    }

    pub(super) fn class_list_length(&mut self, id: NodeId, doc: &Document) -> Result<Value> {
        Ok(Value::Number(self.class_tokens(id, doc)?.len() as f64))
    }

    pub(super) fn token_list_native(
        &mut self,
        name: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let Value::ClassList(id) = receiver else {
            return Err(ScriptError::type_error(
                "incompatible DOMTokenList receiver",
            ));
        };
        if doc.tag(id).is_none() {
            return Err(ScriptError::type_error(
                "incompatible DOMTokenList receiver",
            ));
        }
        if !matches!(name, "add" | "remove" | "contains" | "toggle") {
            return Err(ScriptError::unsupported(
                "DOMTokenList method is not implemented",
            ));
        }
        let variadic = matches!(name, "add" | "remove");
        if !variadic && args.is_empty() {
            return Err(ScriptError::type_error("missing required token"));
        }
        let count = if variadic { args.len() } else { 1 };
        self.work(count + 1)?;
        self.charge(32 + count.saturating_mul(std::mem::size_of::<String>()))?;
        let mut tokens = Vec::new();
        tokens
            .try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("token argument allocation failed"))?;
        for value in &args[..count] {
            tokens.push(self.dom_string(value.clone(), doc)?);
        }
        if name != "contains" {
            for token in &tokens {
                self.work(1 + token.len())?;
                if token.is_empty() {
                    return Err(self.dom_throw("SyntaxError", "empty class token")?);
                }
                if token.chars().any(ascii_space) {
                    return Err(self.dom_throw(
                        "InvalidCharacterError",
                        "class token contains ASCII whitespace",
                    )?);
                }
            }
        }
        // All author conversion has finished. Read the current attribute now.
        let mut classes = self.class_tokens(id, doc)?;
        let mut result = Value::Undefined;
        for token in tokens {
            let scan = classes.iter().fold(1usize, |n, s| {
                n.saturating_add(s.len().min(token.len()) + 1)
            });
            self.work(scan)?;
            let position = classes.iter().position(|s| *s == token);
            if name == "contains" {
                return Ok(Value::Bool(position.is_some()));
            }
            let add = name == "add"
                || name == "toggle"
                    && args
                        .get(1)
                        .filter(|v| **v != Value::Undefined)
                        .map(Value::truthy)
                        .unwrap_or(position.is_none());
            if name == "toggle" {
                result = Value::Bool(add);
                if add == position.is_some() {
                    return Ok(result);
                }
            }
            if add {
                if position.is_none() {
                    if classes.len() == classes.capacity() {
                        let capacity = classes.len().saturating_mul(2).max(4);
                        self.work(1 + classes.len().saturating_mul(3))?;
                        self.charge(
                            (capacity - classes.capacity())
                                .saturating_mul(std::mem::size_of::<String>()),
                        )?;
                        classes
                            .try_reserve_exact(capacity - classes.len())
                            .map_err(|_| ScriptError::resource("class list growth failed"))?;
                    }
                    classes.push(token);
                }
            } else if let Some(position) = position {
                classes.remove(position);
            }
        }
        if classes.is_empty() && doc.attr(id, "class").is_none() {
            return Ok(result);
        }
        let length = classes
            .iter()
            .fold(classes.len().saturating_sub(1), |n, s| {
                n.saturating_add(s.len())
            });
        self.work(1 + length / 8)?;
        self.charge(32 + length.saturating_mul(2))?;
        let mut text = String::new();
        text.try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("class serialization allocation failed"))?;
        for (i, token) in classes.iter().enumerate() {
            if i > 0 {
                text.push(' ');
            }
            text.push_str(token);
        }
        doc.set_attr(id, "class", &text);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check(source: &str) {
        for strict in [false, true] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse(
                "<!doctype html><html><head><title>Before</title></head><body><div id=out>old</div></body></html>",
            );
            runtime
                .execute(
                    include_str!("../../tests/upstream/test262/harness/sta.js"),
                    &mut doc,
                )
                .unwrap();
            runtime
                .execute(
                    include_str!("../../tests/upstream/test262/harness/assert.js"),
                    &mut doc,
                )
                .unwrap();
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            result.unwrap_or_else(|e| panic!("strict={strict}: {e}"));
            assert_eq!(
                (
                    runtime.calls,
                    runtime.stack_units,
                    runtime.eval_depth,
                    runtime.frames.len()
                ),
                (0, 0, 0, 0)
            );
        }
    }

    #[test]
    fn original_dom_conversion_probes_pass_in_both_modes() {
        for line in include_str!("../../tests/fixtures/dom-string-conversion.tsv").lines() {
            let (_, source) = line.split_once('\t').unwrap();
            check(source);
        }
    }

    #[test]
    fn receiver_and_arity_checks_precede_conversion_and_methods_keep_identity() {
        check(
            r#"
            var a=document.createElement('div'),b=document.createElement('p'),hits=0,bad={};
            bad[Symbol.toPrimitive]=function(){hits++;throw 9;};
            var set=a.setAttribute;assert.sameValue(set,b.setAttribute);assert.sameValue(set.name,'setAttribute');assert.sameValue(set.length,2);
            set.call(b,'x','right');assert.sameValue(b.getAttribute('x'),'right');assert.sameValue(a.hasAttribute('x'),false);
            assert.throws(TypeError,function(){set.call({},bad,bad);});
            assert.throws(TypeError,function(){set.call(a,bad);});
            assert.throws(TypeError,function(){set('x','y');});
            assert.sameValue(hits,0);set.apply(b,['y','second']);assert.sameValue(b.getAttribute('y'),'second');
            set.bind(a)('z','bound');assert.sameValue(a.getAttribute('z'),'bound');
            set.call(a,'x','ok',bad);assert.sameValue(hits,0);
            assert.throws(TypeError,function(){document.createTextNode();});
            var create=document.createTextNode;assert.throws(TypeError,function(){create.call(a,bad);});
            assert.sameValue(hits,0);assert.sameValue(create.call(document,undefined).textContent,'undefined');
            assert.throws(TypeError,function(){a.getAttribute();});assert.throws(TypeError,function(){a.removeAttribute();});
            assert.throws(TypeError,function(){document.querySelector();});
            assert.throws(TypeError,function(){document.getElementById.call(a,bad);});assert.sameValue(hits,0);
            assert.throws(TypeError,function(){new a.setAttribute('x','y');});
            var list=a.classList,add=list.add;assert.sameValue(add.length,0);assert.sameValue(list.toggle.length,1);
            assert.throws(TypeError,function(){add.call({},bad);});assert.sameValue(hits,0);
            add.call(b.classList,'other');assert.sameValue(b.className,'other');assert.sameValue(a.className,'');
        "#,
        );
    }

    #[test]
    fn create_text_node_preserves_units_after_conversion_and_distinguishes_defaults() {
        check(
            r#"
            var input = {}, calls = 0;
            input[Symbol.toPrimitive] = function (hint) {
                if (hint !== 'string') throw new Error('hint');
                calls++; return 'A\ud800B\udc00';
            };
            var node = document.createTextNode(input);
            assert.sameValue(calls, 1);
            assert.sameValue(node.data, 'A\ud800B\udc00');
            assert.sameValue(node.length, 4);
            assert.sameValue(node.data.charCodeAt(1), 55296);
            assert.sameValue(node.data.charCodeAt(3), 56320);
            assert.sameValue(document.createTextNode(undefined).data, 'undefined');
            assert.sameValue(document.createTextNode(null).data, 'null');
            assert.sameValue(new Text(undefined).data, '');
            assert.sameValue(new Comment(undefined).data, '');
            assert.sameValue(new Text(null).data, 'null');
            assert.sameValue(new Comment(null).data, 'null');
        "#,
        );
    }

    #[test]
    fn create_text_node_measured_boundaries_publish_no_partial_node() {
        fn setup(grow: bool) -> (Runtime, Document) {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("<p>kept</p>");
            doc.nodes = doc.nodes.into_boxed_slice().into_vec();
            if !grow {
                doc.nodes.try_reserve_exact(1).unwrap();
            }
            runtime.steps = MAX_STEPS;
            (runtime, doc)
        }
        for units in [vec![0x41, 0xd834, 0xdd1e], vec![0xd800, 0x41, 0xdc00]] {
            for grow in [false, true] {
                let args = [Value::String(units.clone().into())];
                let (mut measure, mut doc) = setup(grow);
                let before_heap = measure.allocated;
                measure
                    .dom_native("createTextNode", Value::Document, &args, &mut doc)
                    .unwrap();
                let work = MAX_STEPS - measure.steps;
                let heap = measure.allocated - before_heap;
                for (steps, available, success) in [
                    (work, heap, true),
                    (work - 1, heap, false),
                    (work, heap - 1, false),
                ] {
                    let (mut runtime, mut doc) = setup(grow);
                    let before = format!("{doc:?}");
                    let count = doc.nodes.len();
                    let capacity = doc.nodes.capacity();
                    runtime.steps = steps;
                    runtime.allocated = MAX_HEAP - available;
                    let result =
                        runtime.dom_native("createTextNode", Value::Document, &args, &mut doc);
                    if success {
                        assert_eq!(result.unwrap(), Value::Node(count));
                        assert_eq!(doc.nodes.len(), count + 1);
                        let NodeKind::Text(data) = &doc.nodes[count].kind else {
                            panic!("not Text")
                        };
                        assert_eq!(data.units().collect::<Vec<_>>(), units);
                        assert_eq!(runtime.steps, 0);
                        assert_eq!(runtime.allocated, MAX_HEAP);
                    } else {
                        assert!(result.unwrap_err().is_resource_limit());
                        assert_eq!(format!("{doc:?}"), before);
                        assert_eq!(doc.nodes.capacity(), capacity);
                    }
                    assert_eq!(
                        (
                            runtime.calls,
                            runtime.stack_units,
                            runtime.eval_depth,
                            runtime.frames.len()
                        ),
                        (0, 0, 0, 0)
                    );
                }
            }
        }
    }

    #[test]
    fn string_hooks_reenter_dom_and_abrupt_conversions_preserve_prior_author_effects() {
        check(
            r#"
            var e=document.getElementById('out'),marker={},calls=0,v={};
            v[Symbol.toPrimitive]=function(h){assert.sameValue(this,v);assert.sameValue(h,'string');calls++;e.setAttribute('during','yes');throw marker;};
            try{e.textContent=v;assert(false);}catch(x){assert.sameValue(x,marker);}
            assert.sameValue(e.textContent,'old');assert.sameValue(e.getAttribute('during'),'yes');assert.sameValue(calls,1);
            var old=document.querySelector('title'),newTitle=document.createElement('title'),t={};
            t[Symbol.toPrimitive]=function(h){assert.sameValue(h,'string');old.remove();document.head.appendChild(newTitle);return 'After';};
            document.title=t;assert.sameValue(document.title,'After');assert.sameValue(newTitle.textContent,'After');assert.sameValue(old.textContent,'Before');
            var key={},value={},order='';key[Symbol.toPrimitive]=function(h){order+='k';return 'new';};
            value[Symbol.toPrimitive]=function(h){order+='v';e.setAttribute('side','yes');throw marker;};
            try{e.setAttribute(key,value);assert(false);}catch(x){assert.sameValue(x,marker);}
            assert.sameValue(order,'kv');assert.sameValue(e.hasAttribute('new'),false);assert.sameValue(e.getAttribute('side'),'yes');
            var child=document.createElement('b'),parent=document.createElement('main'),arg={};
            arg[Symbol.toPrimitive]=function(){parent.textContent='during';return 'after';};parent.append(child,arg);
            assert.sameValue(parent.textContent,'duringafter');assert.sameValue(child.parentNode,parent);
            var fallback={toString:function(){return {};},valueOf:function(){return 'fallback';}};
            e.textContent=fallback;assert.sameValue(e.textContent,'fallback');
            var no={toString:1,valueOf:2};assert.throws(TypeError,function(){e.setAttribute('x',no);});
            assert.sameValue(document.createTextNode('A\ud834\udd1eB').textContent,'A\ud834\udd1eB');
        "#,
        );
    }

    #[test]
    fn tokens_convert_before_validation_use_ascii_whitespace_and_read_live_state() {
        check(
            r#"
            var e=document.createElement('div'),list=e.classList,marker={},hit=0,v={};
            v[Symbol.toPrimitive]=function(h){assert.sameValue(h,'string');hit++;throw marker;};
            try{list.add('',v);assert(false);}catch(x){assert.sameValue(x,marker);}
            assert.sameValue(hit,1);assert.sameValue(e.getAttribute('class'),null);
            assert.throws(TypeError,function(){list.add('first',Symbol());});assert.sameValue(e.getAttribute('class'),null);
            try{list.add('first','');assert(false);}catch(x){assert.sameValue(x instanceof DOMException,true);assert.sameValue(x.name,'SyntaxError');assert.sameValue(x.code,12);}
            try{list.add('first','a b');assert(false);}catch(x){assert.sameValue(x.name,'InvalidCharacterError');assert.sameValue(x.code,5);}
            assert.sameValue(e.getAttribute('class'),null);
            list.add();list.remove();assert.sameValue(e.getAttribute('class'),null);
            e.className='  a a\tb\n';assert.sameValue(list.length,2);assert.sameValue(list.contains('a'),true);
            assert.sameValue(list.toggle('a',true),true);assert.sameValue(e.className,'  a a\tb\n');
            assert.sameValue(list.toggle('z',false),false);assert.sameValue(e.className,'  a a\tb\n');
            list.add();assert.sameValue(e.className,'a b');
            assert.sameValue(list.toggle('a',undefined),false);assert.sameValue(e.className,'b');
            assert.sameValue(list.toggle('a',undefined),true);assert.sameValue(e.className,'b a');
            list.add('x\u00a0y','v\u000bw');assert.sameValue(list.length,4);
            assert.sameValue(list.contains(''),false);assert.sameValue(list.contains('a b'),false);
            assert.throws(TypeError,function(){list.contains();});assert.throws(TypeError,function(){list.toggle();});
            var force={};force[Symbol.toPrimitive]=function(){throw marker;};assert.sameValue(list.toggle('forced',force),true);
            var change={};change[Symbol.toPrimitive]=function(){e.className='live';return 'next';};list.add(change);assert.sameValue(e.className,'live next');
            list.remove('live','live');assert.sameValue(e.className,'next');
        "#,
        );
    }

    #[test]
    fn nullable_strings_and_boolean_setters_keep_distinct_conversions() {
        check(
            r#"
            var e=document.getElementById('out'),bad={};bad[Symbol.toPrimitive]=function(){throw 9;};
            e.textContent=null;assert.sameValue(e.textContent,'');e.textContent=undefined;assert.sameValue(e.textContent,'');
            e.innerText=null;assert.sameValue(e.textContent,'');e.innerText=undefined;assert.sameValue(e.textContent,'undefined');
            e.innerHTML=null;assert.sameValue(e.textContent,'');
            e.hidden=bad;assert.sameValue(e.hidden,true);
            var input=document.createElement('input');input.disabled=Symbol();assert.sameValue(input.disabled,true);
            input.checked=bad;assert.sameValue(input.checked,true);
            var disabled=Symbol();e.disabled=disabled;assert.sameValue(e.disabled,disabled);
            e.checked=bad;assert.sameValue(e.checked,bad);
            var textarea=document.createElement('textarea');textarea.value=null;assert.sameValue(textarea.value,'');
            textarea.value=undefined;assert.sameValue(textarea.value,'undefined');
            var s=Symbol();assert.throws(TypeError,function(){textarea.value=s;});assert.sameValue(textarea.value,'undefined');
            assert.throws(TypeError,function(){document.title=s;});assert.sameValue(document.title,'Before');
            e.setAttribute('x',null);assert.sameValue(e.getAttribute('x'),'null');e.setAttribute('x',undefined);assert.sameValue(e.getAttribute('x'),'undefined');
            e.setAttribute('x',-0);assert.sameValue(e.getAttribute('x'),'0');e.setAttribute('x',1e21);assert.sameValue(e.getAttribute('x'),'1e+21');
        "#,
        );
    }

    #[test]
    fn converted_storage_and_callback_limits_refuse_before_host_mutation() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<div id=out>old</div>");
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .dom_string(Value::String("abc".into()), &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        runtime.allocated = 0;
        runtime.steps = 0;
        assert!(
            runtime
                .dom_string(Value::String("abc".into()), &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, 0);
        for body in [
            "while(true){}",
            "document.getElementById('out').textContent=v;return 'x';",
        ] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("<div id=out>old</div>");
            let source = format!(
                "var caught=false,v={{}};v[Symbol.toPrimitive]=function(){{{body}}};try{{document.getElementById('out').textContent=v;}}catch(e){{caught=true;}}"
            );
            assert!(
                runtime
                    .execute(&source, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(
                runtime.environments[0].bindings["caught"].value,
                Value::Bool(false)
            );
            assert_eq!(doc.text_content(doc.query_selector("#out").unwrap()), "old");
            assert_eq!(
                (
                    runtime.calls,
                    runtime.stack_units,
                    runtime.eval_depth,
                    runtime.frames.len()
                ),
                (0, 0, 0, 0)
            );
        }
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<div id=out></div>");
        let id = doc.query_selector("#out").unwrap();
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .dom_native(
                    "append",
                    Value::Node(id),
                    &[Value::String("new".into())],
                    &mut doc
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(doc.nodes[id].children.is_empty());
    }
}
