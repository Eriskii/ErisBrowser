//! Lazy ordinary own descriptors for Document and Node identities.
//!
//! The storage object is never a public receiver or a prototype. In particular,
//! owning a descriptor does not admit host integrity/prototype operations.
use super::*;
use property_keys::HostKey;

#[cfg(test)]
mod tests;

pub(super) fn host(receiver: &Value) -> Option<HostKey> {
    match receiver {
        Value::Document => Some(HostKey::Document),
        Value::Node(id) => Some(HostKey::Node(*id)),
        _ => None,
    }
}

fn tree(count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let mut capacity = count.saturating_add(1) / 2;
    let mut levels = 1;
    while capacity >= 6 {
        capacity /= 6;
        levels += 1;
    }
    (count.min(11 * levels), levels)
}
fn search(count: usize, key: &PropertyKey) -> usize {
    tree(count).0.saturating_mul(1 + key.byte_len() / 2)
}
fn moves(count: usize) -> usize {
    28 * (tree(count).1 + 1)
}
fn nodes<K, V>(count: usize) -> usize {
    (tree(count).1 + 1).saturating_mul(
        16 * (std::mem::size_of::<K>() + std::mem::size_of::<V>())
            + 32 * std::mem::size_of::<usize>()
            + 64,
    )
}
fn overflow() -> ScriptError {
    ScriptError::resource("DOM own-property storage overflow")
}

impl Runtime {
    pub(super) fn dom_own_object(&mut self, receiver: &Value) -> Result<Option<usize>> {
        let Some(host) = host(receiver) else {
            return Ok(None);
        };
        self.work(1 + tree(self.host_symbol_objects.len()).0)?;
        Ok(self.host_symbol_objects.get(&host).copied())
    }

    pub(super) fn read_own_property_key(
        &mut self,
        receiver: &Value,
        key: &PropertyKey,
    ) -> Result<Option<Property>> {
        if host(receiver).is_none() {
            return Ok(self.own_property_key(receiver, key));
        }
        self.dom_ordinary_key(receiver, key)?;
        let Some(id) = self.dom_own_object(receiver)? else {
            return Ok(None);
        };
        self.work(1 + search(self.objects[id].values.len(), key))?;
        Ok(self.objects[id].values.get(key).cloned())
    }

    pub(super) fn read_own_property(
        &mut self,
        receiver: &Value,
        key: &JsString,
    ) -> Result<Option<Property>> {
        if host(receiver).is_none() {
            return Ok(self.own_property(receiver, key));
        }
        self.read_own_property_key(receiver, &PropertyKey::String(key.clone()))
    }

    fn dom_ordinary_key(&mut self, receiver: &Value, key: &PropertyKey) -> Result<()> {
        // HTML Document's unforgeable location is not implemented. Do not
        // silently replace its required own descriptor with an ordinary expando.
        if matches!(receiver, Value::Document) {
            self.work(9)?;
            if let Some(key) = key.as_string()
                && key.units() == [108, 111, 99, 97, 116, 105, 111, 110]
            {
                return Err(ScriptError::unsupported(
                    "Document unforgeable location is not implemented",
                ));
            }
        }
        Ok(())
    }

    pub(super) fn dom_define_own(
        &mut self,
        receiver: &Value,
        key: &PropertyKey,
        desc: PropertyDescriptor,
    ) -> Result<bool> {
        self.dom_ordinary_key(receiver, key)?;
        let host = host(receiver).expect("DOM receiver");
        let id = self.dom_own_object(receiver)?;
        let count = id.map_or(0, |id| self.objects[id].values.len());
        self.work(9 + search(count, key))?;
        let current = id.and_then(|id| self.objects[id].values.get(key).cloned());
        // Descriptor conversion has already completed. Snapshot live state now;
        // no author code runs during validation, reservation or publication.
        if let Some(old) = &current
            && !old.configurable
        {
            if desc.configurable == Some(true)
                || desc.enumerable.is_some_and(|flag| flag != old.enumerable)
            {
                return Ok(false);
            }
            match &old.value {
                PropertyValue::Data { value, writable } => {
                    if desc.accessor() || (!writable && desc.writable == Some(true)) {
                        return Ok(false);
                    }
                    if !writable
                        && let Some(next) = &desc.value
                        && !self.object_is_values(value, next)?
                    {
                        return Ok(false);
                    }
                }
                PropertyValue::Accessor { get, set } => {
                    if desc.data() {
                        return Ok(false);
                    }
                    if let Some(next) = &desc.get
                        && !self.object_is_values(get, next)?
                    {
                        return Ok(false);
                    }
                    if let Some(next) = &desc.set
                        && !self.object_is_values(set, next)?
                    {
                        return Ok(false);
                    }
                }
            }
        }
        let fresh = current.is_none();
        let mut property =
            current.unwrap_or_else(|| Property::data(Value::Undefined, false, false, false));
        if desc.accessor() && matches!(property.value, PropertyValue::Data { .. }) {
            property.value = PropertyValue::Accessor {
                get: Value::Undefined,
                set: Value::Undefined,
            };
        } else if desc.data() && matches!(property.value, PropertyValue::Accessor { .. }) {
            property.value = PropertyValue::Data {
                value: Value::Undefined,
                writable: false,
            };
        }
        if let Some(flag) = desc.enumerable {
            property.enumerable = flag;
        }
        if let Some(flag) = desc.configurable {
            property.configurable = flag;
        }
        match &mut property.value {
            PropertyValue::Data { value, writable } => {
                if let Some(next) = desc.value {
                    *value = next;
                }
                if let Some(next) = desc.writable {
                    *writable = next;
                }
            }
            PropertyValue::Accessor { get, set } => {
                if let Some(next) = desc.get {
                    *get = next;
                }
                if let Some(next) = desc.set {
                    *set = next;
                }
            }
        }
        // Actual-type tree allocations and fixed entry/edge moves are separate
        // from UTF-16 search work. Retained key/value payloads clone handles.
        self.work(search(count, key) + if fresh { moves(count) } else { 0 })?;
        if fresh {
            self.charge(nodes::<PropertyKey, Property>(count))?;
        }
        let order_len = id.map_or(0, |id| self.objects[id].order.len());
        let order_cap = id.map_or(0, |id| self.objects[id].order.capacity());
        let order_next = if fresh && order_len == order_cap {
            Some(
                order_cap
                    .checked_mul(2)
                    .map(|n| n.max(4))
                    .ok_or_else(overflow)?,
            )
        } else {
            None
        };
        if let Some(capacity) = order_next {
            self.work(1 + 2 * order_len)?;
            self.charge(
                capacity
                    .checked_mul(std::mem::size_of::<PropertyKey>())
                    .ok_or_else(overflow)?,
            )?;
        }
        let arena_next = if id.is_none() && self.objects.len() == self.objects.capacity() {
            Some(
                self.objects
                    .capacity()
                    .checked_mul(2)
                    .map(|n| n.max(4))
                    .ok_or_else(overflow)?,
            )
        } else {
            None
        };
        if id.is_none() {
            let hosts = self.host_symbol_objects.len();
            self.work(4 + tree(hosts).0 + moves(hosts))?;
            self.charge(std::mem::size_of::<ScriptObject>() + nodes::<HostKey, usize>(hosts))?;
            if let Some(capacity) = arena_next {
                self.work(1 + 2 * self.objects.len())?;
                self.charge(
                    capacity
                        .checked_mul(std::mem::size_of::<ScriptObject>())
                        .ok_or_else(overflow)?,
                )?;
            }
        }
        // All admission precedes fallible allocation. No descriptor or host-map
        // entry changes on failure; successfully reserved capacity may remain.
        if let Some(capacity) = arena_next {
            self.objects
                .try_reserve_exact(capacity - self.objects.len())
                .map_err(|_| ScriptError::resource("DOM own-object allocation failed"))?;
        }
        if let Some(id) = id {
            if let Some(capacity) = order_next {
                self.objects[id]
                    .order
                    .try_reserve_exact(capacity - order_len)
                    .map_err(|_| ScriptError::resource("DOM own-key order allocation failed"))?;
            }
            let bag = &mut self.objects[id];
            if fresh {
                bag.order.push(key.clone());
            }
            bag.values.insert(key.clone(), property);
        } else {
            let mut bag = ScriptObject::default();
            bag.order
                .try_reserve_exact(order_next.unwrap())
                .map_err(|_| ScriptError::resource("DOM own-key order allocation failed"))?;
            bag.order.push(key.clone());
            bag.values.insert(key.clone(), property);
            let id = self.objects.len();
            self.objects.push(bag);
            self.host_symbol_objects.insert(host, id);
        }
        Ok(true)
    }

    pub(super) fn dom_delete_own(&mut self, receiver: &Value, key: &PropertyKey) -> Result<bool> {
        self.dom_ordinary_key(receiver, key)?;
        let Some(id) = self.dom_own_object(receiver)? else {
            return Ok(true);
        };
        let count = self.objects[id].values.len();
        self.work(1 + search(count, key))?;
        let Some(property) = self.objects[id].values.get(key) else {
            return Ok(true);
        };
        if !property.configurable {
            return Ok(false);
        }
        self.work(
            // Deletion can rebalance both the reached leaf and its parent.
            // Two insertion-move allowances per level cover merging/rotation,
            // in addition to the separately paid order compaction below.
            search(count, key)
                + 2 * moves(count)
                + self.objects[id]
                    .order
                    .len()
                    .saturating_mul(2 + key.byte_len() / 2),
        )?;
        self.objects[id].remove(key);
        Ok(true)
    }

    pub(super) fn dom_own_get_utf8(
        &mut self,
        receiver: &Value,
        key: &str,
        doc: &mut Document,
    ) -> Result<Option<Value>> {
        let Some(id) = self.dom_own_object(receiver)? else {
            return Ok(None);
        };
        if self.objects[id].values.is_empty() {
            return Ok(None);
        }
        // Direct get has UTF-8 input. Bound the exact-reserve temporary by its
        // byte length, then pay decoding, final Rc copy and both payloads before
        // constructing the additional key. The common empty-bag path copies none.
        self.work(1usize.saturating_add(key.len().saturating_mul(3)))?;
        self.charge(64usize.saturating_add(key.len().saturating_mul(4)))?;
        let mut units = Vec::new();
        units
            .try_reserve_exact(key.len())
            .map_err(|_| ScriptError::resource("DOM own-key conversion allocation failed"))?;
        units.extend(key.encode_utf16());
        self.dom_own_get(receiver, &JsString::from(units), doc)
    }

    pub(super) fn dom_own_get(
        &mut self,
        receiver: &Value,
        key: &JsString,
        doc: &mut Document,
    ) -> Result<Option<Value>> {
        let Some(property) = self.read_own_property(receiver, key)? else {
            return Ok(None);
        };
        Ok(Some(match property.value {
            PropertyValue::Data { value, .. } => value,
            PropertyValue::Accessor {
                get: Value::Undefined,
                ..
            } => Value::Undefined,
            PropertyValue::Accessor { get, .. } => {
                self.call(get, Vec::new(), receiver.clone(), doc)?
            }
        }))
    }

    pub(super) fn dom_call_setter(
        &mut self,
        setter: Value,
        value: Value,
        receiver: Value,
        doc: &mut Document,
    ) -> Result<()> {
        self.tick()?;
        self.charge(std::mem::size_of::<Value>())?;
        let mut arguments = Vec::new();
        arguments
            .try_reserve_exact(1)
            .map_err(|_| ScriptError::resource("DOM setter argument allocation failed"))?;
        arguments.push(value);
        self.call(setter, arguments, receiver, doc)?;
        Ok(())
    }

    pub(super) fn dom_set_ordinary(
        &mut self,
        receiver: &Value,
        key: &JsString,
        value: &Value,
        strict: bool,
        doc: &mut Document,
    ) -> Result<bool> {
        let own = self.read_own_property(receiver, key)?;
        if own.is_none() {
            match self.dom_attribute_key(receiver, key, doc)? {
                Some(AttributeWrite::Setter) => return Ok(false),
                Some(AttributeWrite::Readonly) => {
                    Self::failed_write(strict)?;
                    return Ok(true);
                }
                Some(AttributeWrite::Unsupported | AttributeWrite::Unavailable) => {
                    return Err(ScriptError::unsupported(
                        "native DOM attribute setter is not implemented",
                    ));
                }
                _ => {}
            }
        }
        let property = match own {
            Some(property) => Some(property),
            None => self.find_property(receiver, key)?,
        };
        if let Some(property) = property {
            match property.value {
                PropertyValue::Accessor {
                    set: Value::Undefined,
                    ..
                }
                | PropertyValue::Data {
                    writable: false, ..
                } => {
                    Self::failed_write(strict)?;
                    return Ok(true);
                }
                PropertyValue::Accessor { set, .. } => {
                    self.dom_call_setter(set, value.clone(), receiver.clone(), doc)?;
                    return Ok(true);
                }
                _ => {}
            }
        }
        let present = self.read_own_property(receiver, key)?.is_some();
        let desc = if present {
            PropertyDescriptor {
                value: Some(value.clone()),
                ..PropertyDescriptor::default()
            }
        } else {
            PropertyDescriptor::data_property(value.clone(), true, true, true)
        };
        if !self.dom_define_own(receiver, &PropertyKey::String(key.clone()), desc)? {
            Self::failed_write(strict)?;
        }
        Ok(true)
    }

    fn dom_str_in(&mut self, key: &str, names: &[&str]) -> Result<bool> {
        for name in names {
            self.tick()?;
            if key.len() == name.len() {
                self.work(key.len())?;
                if key == *name {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn dom_attribute_key(
        &mut self,
        receiver: &Value,
        key: &JsString,
        doc: &Document,
    ) -> Result<Option<AttributeWrite>> {
        // All names in the existing native fallback inventory are ASCII and
        // at most documentElement.len(). Longer/non-ASCII keys need no copy.
        let mut ascii = [0u8; "documentElement".len()];
        self.tick()?;
        if key.len() > ascii.len() {
            return Ok(None);
        }
        self.work(key.len())?;
        for (out, unit) in ascii.iter_mut().zip(key.units()) {
            if *unit > 127 {
                return Ok(None);
            }
            *out = *unit as u8;
        }
        self.dom_attribute(
            receiver,
            std::str::from_utf8(&ascii[..key.len()]).unwrap(),
            doc,
        )
    }

    pub(super) fn dom_attribute_get(
        &mut self,
        receiver: &Value,
        key: &str,
        doc: &Document,
    ) -> Result<bool> {
        match self.dom_attribute(receiver, key, doc)? {
            Some(AttributeWrite::Unavailable) => Err(ScriptError::unsupported(
                "native DOM attribute getter is not implemented",
            )),
            value => Ok(value == Some(AttributeWrite::Absent)),
        }
    }

    fn dom_attribute(
        &mut self,
        receiver: &Value,
        key: &str,
        doc: &Document,
    ) -> Result<Option<AttributeWrite>> {
        use AttributeRule::*;
        use AttributeWrite::{Absent, Readonly, Setter, Unavailable};
        let mut rule = None;
        for (name, candidate) in ATTRIBUTES {
            if self.dom_str_in(key, &[*name])? {
                rule = Some(*candidate);
                break;
            }
        }
        let Some(rule) = rule else {
            return Ok(None);
        };
        self.work(4)?;
        let (element, html, tag, parent) = match receiver {
            Value::Node(id) => {
                let Some(node) = doc.nodes.get(*id) else {
                    return Err(ScriptError::new("invalid DOM node"));
                };
                let element = matches!(node.kind, NodeKind::Element(_));
                (
                    element,
                    doc.namespace(*id) == Some(Namespace::Html),
                    doc.tag(*id).unwrap_or(""),
                    element || matches!(node.kind, NodeKind::DocumentFragment { .. }),
                )
            }
            _ => (false, false, "", false),
        };
        let document = matches!(receiver, Value::Document);
        let disposition = match rule {
            Node(write) if matches!(receiver, Value::Node(_)) => write,
            // The singleton Document's inherited Node accessors are a separate
            // missing binding; do not silently create an expando for them.
            Node(_) if document => Unavailable,
            Element(write) if element => write,
            Html(write) if html || (document && key == "title") => write,
            Tags(tags, write) if html && self.dom_str_in(tag, tags)? => write,
            Parent if parent => Readonly,
            Parent if document => Unavailable,
            BaseUri if document => Readonly,
            BaseUri if matches!(receiver, Value::Node(_)) => Unavailable,
            HtmlType if html => {
                if self.dom_str_in(tag, &["fieldset", "output", "select", "textarea"])? {
                    Readonly
                } else if self.dom_str_in(
                    tag,
                    &[
                        "a", "button", "embed", "input", "link", "object", "ol", "script",
                        "source", "li", "param", "style", "ul",
                    ],
                )? {
                    Setter
                } else {
                    Absent
                }
            }
            Document(write) if document => write,
            Event if document || element => Setter,
            _ => Absent,
        };
        Ok(Some(disposition))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AttributeWrite {
    Absent,
    Setter,
    Readonly,
    Unsupported,
    Unavailable,
}
#[derive(Clone, Copy)]
enum AttributeRule {
    Node(AttributeWrite),
    Element(AttributeWrite),
    Html(AttributeWrite),
    Tags(&'static [&'static str], AttributeWrite),
    Parent,
    HtmlType,
    BaseUri,
    Document(AttributeWrite),
    Event,
}
// Existing native attribute inventory, classified by its actual defining
// interface. This is not a complete IDL inventory: broader missing attributes
// (for example input.name, meta.content and the legacy document.all collection)
// remain separate binding gaps. Method names are ordinary shadows.
const ATTRIBUTES: &[(&str, AttributeRule)] = {
    use AttributeRule::*;
    use AttributeWrite::{Readonly as R, Setter as S, Unsupported as U};
    &[
        ("textContent", Node(S)),
        ("nodeName", Node(R)),
        ("nodeType", Node(R)),
        ("parentNode", Node(R)),
        ("parentElement", Node(R)),
        ("firstChild", Node(R)),
        ("lastChild", Node(R)),
        ("childNodes", Node(R)),
        ("namespaceURI", Element(R)),
        ("tagName", Element(R)),
        ("children", Parent),
        ("id", Element(S)),
        ("className", Element(S)),
        ("innerHTML", Element(S)),
        ("outerHTML", Element(U)),
        ("style", Element(U)),
        ("classList", Element(U)),
        ("innerText", Html(S)),
        ("title", Html(S)),
        ("hidden", Html(S)),
        ("open", Tags(&["details"], S)),
        ("name", Tags(&["details"], S)),
        ("content", Tags(&["template"], R)),
        (
            "value",
            Tags(
                &[
                    "input", "textarea", "button", "option", "select", "output", "li", "data",
                    "meter", "progress", "param",
                ],
                S,
            ),
        ),
        ("href", Tags(&["a", "area", "base", "link"], S)),
        (
            "src",
            Tags(
                &[
                    "audio", "video", "embed", "iframe", "img", "input", "script", "source",
                    "track", "frame",
                ],
                S,
            ),
        ),
        ("type", HtmlType),
        ("checked", Tags(&["input"], S)),
        (
            "disabled",
            Tags(
                &[
                    "button", "fieldset", "input", "optgroup", "option", "select", "textarea",
                    "link", "style",
                ],
                S,
            ),
        ),
        ("defaultView", Document(R)),
        ("URL", Document(R)),
        ("documentURI", Document(R)),
        ("baseURI", BaseUri),
        ("characterSet", Document(R)),
        ("charset", Document(R)),
        ("inputEncoding", Document(R)),
        ("compatMode", Document(R)),
        ("head", Document(R)),
        ("body", Document(U)),
        ("documentElement", Document(R)),
        ("readyState", Document(R)),
        ("onclick", Event),
        ("ontoggle", Event),
        ("ondblclick", Event),
        ("oninput", Event),
        ("onbeforeinput", Event),
        ("onchange", Event),
        ("onsubmit", Event),
        ("onreset", Event),
        ("onkeydown", Event),
        ("onkeyup", Event),
        ("onkeypress", Event),
        ("onfocus", Event),
        ("onblur", Event),
        ("onfocusin", Event),
        ("onfocusout", Event),
        ("onmousedown", Event),
        ("onmouseup", Event),
        ("onmousemove", Event),
        ("onmouseenter", Event),
        ("onmouseleave", Event),
        ("onmouseover", Event),
        ("onmouseout", Event),
        ("onwheel", Event),
        ("oncontextmenu", Event),
        ("onload", Event),
        ("onerror", Event),
        ("onscroll", Event),
        ("onresize", Event),
        ("onunload", Event),
        ("ontouchstart", Event),
        ("ontouchmove", Event),
        ("ontouchend", Event),
    ]
};
