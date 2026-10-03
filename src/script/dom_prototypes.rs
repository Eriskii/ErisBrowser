//! Interface identity and prototype metadata for represented DOM nodes.
//! Other interface members remain explicitly incomplete; node brands are not
//! inferred from mutable JavaScript prototype membership.
use super::*;
use std::collections::btree_map::Entry;

mod interfaces;
mod node_constants;
#[cfg(test)]
mod tests;

#[cfg(test)]
#[test]
fn dom_event_target_tag_does_not_mask_abort_signal() {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut doc = Document::parse("<p>kept</p>");
        let source = format!(
            r#"{}
            (function () {{
                function check(value) {{ if (!value) throw new Error('event tag'); }}
                var key = Symbol.toStringTag;
                var own = Object.getOwnPropertyDescriptor(AbortSignal.prototype, key);
                var base = Object.getOwnPropertyDescriptor(EventTarget.prototype, key);
                var signal = (new AbortController()).signal;
                check(own.value === 'AbortSignal' && own.writable === false &&
                      own.enumerable === false && own.configurable === true);
                check(own.get === undefined && own.set === undefined);
                check(Object.prototype.toString.call(signal) === '[object AbortSignal]');
                check(Object.prototype.toString.call(Object.create(AbortSignal.prototype)) ===
                      '[object AbortSignal]');
                try {{
                    Object.defineProperty(EventTarget.prototype, key, {{value: 'ChangedBase'}});
                    check(Object.prototype.toString.call(signal) === '[object AbortSignal]');
                    check(delete AbortSignal.prototype[key]);
                    check(Object.prototype.toString.call(signal) === '[object ChangedBase]');
                }} finally {{
                    Object.defineProperty(AbortSignal.prototype, key, own);
                    Object.defineProperty(EventTarget.prototype, key, base);
                }}
                check(Object.prototype.toString.call(signal) === '[object AbortSignal]');
                return true;
            }})()
            "#,
            if strict { "'use strict';" } else { "" }
        );
        assert_eq!(
            runtime.execute(&source, &mut doc).unwrap(),
            Value::Bool(true)
        );
    }
}

use interfaces::{ConstructorKind, INTERFACES};
pub(super) const PREFIX: &str = "DOM.Interface.";
// Five unscopables objects: ParentNode's three including interfaces, plus
// CharacterData and DocumentType's ChildNode lists (Element combines both).
pub(super) const BOOTSTRAP_OBJECTS: usize = 352
    + 2 * (INTERFACES.len() - 1)
    + 5
    + processing_instruction::METADATA_OBJECTS
    + node_connected::METADATA_OBJECTS
    + node_data::METADATA_OBJECTS
    + node_normalize::METADATA_OBJECTS
    + node_predicates::METADATA_OBJECTS
    + node_root::METADATA_OBJECTS
    + node_equality::METADATA_OBJECTS
    + node_position::METADATA_OBJECTS
    + document_title::METADATA_OBJECTS
    + text_operations::METADATA_OBJECTS;

#[derive(Default)]
pub(super) struct State {
    // Eager interface objects/prototypes, indexed by the static inventory.
    // Published together with all global bindings only after bootstrap succeeds.
    records: Vec<InterfaceRecord>,
    // Only genuine constructed nodes with an explicit different prototype
    // need an entry. Own descriptor bags remain independent.
    overrides: BTreeMap<NodeId, Value>,
}

pub(super) fn is_interface_name(name: &str) -> bool {
    name.starts_with(PREFIX)
}
fn tree_bound(count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let mut capacity = count.saturating_add(1) / 2;
    let mut height = 1;
    while capacity >= 6 {
        capacity /= 6;
        height += 1;
    }
    (count.min(11 * height), height)
}
fn search(count: usize, length: usize) -> usize {
    tree_bound(count).0.saturating_mul(1 + length / 8)
}
fn moves(count: usize) -> usize {
    if count < 11 {
        count + 8
    } else {
        28 * (tree_bound(count).1 + 1)
    }
}
fn node_bytes<K, V>() -> usize {
    16 * (std::mem::size_of::<K>() + std::mem::size_of::<V>())
        + 32 * std::mem::size_of::<usize>()
        + 64
}
fn insert_bytes<K, V>(count: usize) -> usize {
    if count == 0 {
        node_bytes::<K, V>()
    } else if count < 11 {
        0
    } else {
        (tree_bound(count).1 + 1) * node_bytes::<K, V>()
    }
}

struct InterfaceRecord {
    prototype: usize,
    constructor: Value,
    order: u64,
}
const PARENT_UNSCOPABLES: &[&str] = &["prepend", "append", "replaceChildren"];
const CHILD_UNSCOPABLES: &[&str] = &["before", "after", "replaceWith", "remove"];
const ELEMENT_UNSCOPABLES: &[&str] = &[
    "prepend",
    "append",
    "replaceChildren",
    "before",
    "after",
    "replaceWith",
    "remove",
];
const NODE_CONSTANTS: &[(&str, u32)] = &[
    ("ELEMENT_NODE", 1),
    ("ATTRIBUTE_NODE", 2),
    ("TEXT_NODE", 3),
    ("CDATA_SECTION_NODE", 4),
    ("ENTITY_REFERENCE_NODE", 5),
    ("ENTITY_NODE", 6),
    ("PROCESSING_INSTRUCTION_NODE", 7),
    ("COMMENT_NODE", 8),
    ("DOCUMENT_NODE", 9),
    ("DOCUMENT_TYPE_NODE", 10),
    ("DOCUMENT_FRAGMENT_NODE", 11),
    ("NOTATION_NODE", 12),
    ("DOCUMENT_POSITION_DISCONNECTED", 1),
    ("DOCUMENT_POSITION_PRECEDING", 2),
    ("DOCUMENT_POSITION_FOLLOWING", 4),
    ("DOCUMENT_POSITION_CONTAINS", 8),
    ("DOCUMENT_POSITION_CONTAINED_BY", 16),
    ("DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC", 32),
];
fn interface_binding(value: Value, order: u64) -> Binding {
    Binding {
        value,
        accessor: None,
        mutable: true,
        initialized: true,
        strict_immutable: false,
        global_property: true,
        global_order: order,
        enumerable: false,
        deletable: true,
    }
}

// Byte-key maps only. PropertyKey metadata uses full UTF-16 comparison work.
struct BulkPlan {
    work: usize,
    bytes: usize,
}
fn bulk_plan<K: Ord + AsRef<str>, V>(old: &BTreeMap<K, V>, new: &[(K, V)]) -> Result<BulkPlan> {
    let total = old
        .len()
        .checked_add(new.len())
        .ok_or_else(|| ScriptError::resource("DOM metadata map length overflow"))?;
    let max = old
        .keys()
        .map(|key| key.as_ref().len())
        .chain(new.iter().map(|(key, _)| key.as_ref().len()))
        .max()
        .unwrap_or(0);
    let mut height = 0usize;
    let mut remaining = total;
    while remaining >= 12 {
        remaining /= 12;
        height += 1;
    }
    let old_nodes = if old.is_empty() {
        0
    } else {
        1 + (old.len() - 1) / 5
    };
    let allocated_nodes = if total == 0 {
        0
    } else {
        total / 11 + height + 1
    };
    let overflow = || ScriptError::resource("DOM metadata map work overflow");
    let n = total;
    let h = height;
    let o = old.len();
    let m = new.len();
    // Staged ordering, a single collision-detecting consuming merge,
    // then sorted-run detection and adjacent dedup.
    let comparisons = m
        .saturating_sub(1)
        .checked_add(n.saturating_sub(1).checked_mul(3).ok_or_else(overflow)?)
        .ok_or_else(overflow)?;
    // Fixed-size pair moves and edge/header transitions, not byte copies.
    // Four per old entry/node covers the single consuming merge traversal.
    // The earlier length scan has its own debit.
    let terms = [
        o.checked_add(old_nodes).and_then(|v| v.checked_mul(4)),
        n.checked_mul(6), // merge entry/iterator moves
        n.checked_mul(8), // fresh-tree pair writes and length transitions
        h.checked_mul(2)
            .and_then(|v| v.checked_add(2))
            .and_then(|v| v.checked_mul(4))
            .and_then(|v| v.checked_mul(n / 12)),
        h.checked_mul(64),              // final right-border repairs
        allocated_nodes.checked_mul(8), // allocated-node header setup
        h.checked_add(1).and_then(|v| v.checked_mul(16)),
        (1 + max / 8).checked_mul(comparisons),
    ];
    let work = terms.into_iter().try_fold(64usize, |sum, term| {
        sum.checked_add(term.ok_or_else(overflow)?)
            .ok_or_else(overflow)
    })?;
    // Full sort scratch despite ascending input; direct, unadvanced Vec
    // IntoIter collection reuses the already admitted merged-vector block.
    let bytes = total
        .max(48)
        .checked_mul(std::mem::size_of::<(K, V)>())
        .and_then(|n| n.checked_add(allocated_nodes.checked_mul(node_bytes::<K, V>())?))
        .ok_or_else(|| ScriptError::resource("DOM metadata map storage overflow"))?;
    Ok(BulkPlan { work, bytes })
}
fn validate_staged<K: Ord, V>(new: &[(K, V)]) -> Result<()> {
    if new.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(ScriptError::resource(
            "DOM metadata staging is not strictly sorted",
        ));
    }
    Ok(())
}
fn merged_map<K: Ord, V>(
    old: BTreeMap<K, V>,
    new: Vec<(K, V)>,
    mut merged: Vec<(K, V)>,
) -> Result<BTreeMap<K, V>> {
    let mut old = old.into_iter().peekable();
    let mut new = new.into_iter().peekable();
    while let (Some(a), Some(b)) = (old.peek(), new.peek()) {
        match a.0.cmp(&b.0) {
            std::cmp::Ordering::Less => merged.push(old.next().unwrap()),
            std::cmp::Ordering::Greater => merged.push(new.next().unwrap()),
            std::cmp::Ordering::Equal => {
                return Err(ScriptError::resource(
                    "DOM metadata collides with an existing intrinsic",
                ));
            }
        }
    }
    merged.extend(old);
    merged.extend(new);
    Ok(merged.into_iter().collect())
}

impl Runtime {
    pub(super) fn dom_proto_text(&mut self, text: &str) -> Result<JsString> {
        // Static ASCII inputs: installer text and the fixed root-options key.
        // Two copies are paid: exact Vec then Rc.
        self.work(8 + 2 * text.len())?;
        self.charge(64 + 4 * text.len())?;
        let mut units = Vec::new();
        units
            .try_reserve_exact(text.len())
            .map_err(|_| ScriptError::resource("DOM prototype string allocation failed"))?;
        units.extend(text.bytes().map(u16::from));
        Ok(units.into())
    }
    fn dom_interface_value(&mut self, name: &str, properties: usize) -> Result<Value> {
        let properties = std::num::NonZeroUsize::new(properties)
            .ok_or_else(|| ScriptError::resource("DOM interface metadata ID is zero"))?;
        let length = PREFIX.len() + name.len();
        self.work(8 + length)?;
        self.charge(std::mem::size_of::<Native>() + 32 + length)?;
        let mut full = String::new();
        full.try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("DOM interface name allocation failed"))?;
        full.push_str(PREFIX);
        full.push_str(name);
        Ok(Value::Native(Rc::new(Native {
            properties: Some(properties),
            name: full,
            receiver: Value::Window,
        })))
    }
    pub(super) fn dom_proto_object(
        &mut self,
        prototype: Option<Value>,
        capacity: usize,
    ) -> Result<usize> {
        self.work(8)?;
        self.charge(
            72 + std::mem::size_of::<Option<AbortSlot>>() + std::mem::size_of::<Option<f64>>(),
        )?;
        if self.objects.len() == self.objects.capacity() {
            return Err(ScriptError::resource(
                "DOM prototype arena reservation exhausted",
            ));
        }
        self.work(1)?;
        self.charge(capacity * std::mem::size_of::<PropertyKey>())?;
        let mut bag = ScriptObject {
            prototype,
            ..ScriptObject::default()
        };
        bag.order
            .try_reserve_exact(capacity)
            .map_err(|_| ScriptError::resource("DOM metadata order allocation failed"))?;
        let id = self.objects.len();
        self.objects.push(bag);
        Ok(id)
    }
    fn dom_proto_property(
        &mut self,
        owner: usize,
        key: PropertyKey,
        property: Property,
    ) -> Result<()> {
        let count = self.objects[owner].values.len();
        let length = key.as_string().map_or(0, JsString::len);
        let order = &self.objects[owner].order;
        let grow = order.len() == order.capacity();
        let capacity = order.len() + 1;
        self.work(
            tree_bound(count).0.saturating_mul(1 + length)
                + moves(count)
                + if grow { 1 + 2 * count } else { 1 },
        )?;
        self.charge(
            256 + 4 * length
                + insert_bytes::<PropertyKey, Property>(count)
                + if grow {
                    capacity * std::mem::size_of::<PropertyKey>()
                } else {
                    0
                },
        )?;
        let bag = &mut self.objects[owner];
        if grow {
            bag.order
                .try_reserve_exact(1)
                .map_err(|_| ScriptError::resource("DOM prototype order allocation failed"))?;
        }
        let Entry::Vacant(entry) = bag.values.entry(key.clone()) else {
            return Err(ScriptError::resource("duplicate DOM prototype member"));
        };
        bag.order.push(key);
        entry.insert(property);
        Ok(())
    }
    pub(super) fn dom_proto_named(
        &mut self,
        owner: usize,
        key: &str,
        property: Property,
    ) -> Result<()> {
        let key = self.dom_proto_text(key)?.into();
        self.dom_proto_property(owner, key, property)
    }
    pub(super) fn dom_proto_id(&mut self, name: &str) -> Result<usize> {
        if self.dom_prototypes.records.is_empty() {
            self.work(search(self.prototypes.len(), name.len()))?;
            return self
                .prototypes
                .get(name)
                .copied()
                .ok_or_else(|| ScriptError::resource("DOM prototype parent missing"));
        }
        self.work(interfaces::interface_lookup_work(name.len()) + 2)?;
        interfaces::interface_index(name)
            .and_then(|index| self.dom_prototypes.records.get(index))
            .map(|record| record.prototype)
            .ok_or_else(|| ScriptError::resource("DOM prototype index missing"))
    }
    pub(super) fn initialize_dom_prototypes(&mut self) -> Result<()> {
        let count = INTERFACES.len();
        self.work(8 + count)?;
        self.charge(count * std::mem::size_of::<InterfaceRecord>())?;
        let mut records = Vec::<InterfaceRecord>::new();
        records
            .try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("DOM interface records allocation failed"))?;
        let keys: [PropertyKey; 4] = [
            self.dom_proto_text("name")?.into(),
            self.dom_proto_text("length")?.into(),
            self.dom_proto_text("prototype")?.into(),
            self.dom_proto_text("constructor")?.into(),
        ];
        let tag_key = self.well_known_key("toStringTag");
        let unscopables_key = self.well_known_key("unscopables");
        let first_order = self.next_global_order;
        let mut new_count = 0usize;
        for (interface_index, interface) in INTERFACES.iter().enumerate() {
            self.work(16)?;
            if interface.name == "EventTarget" {
                let prototype = self.dom_proto_id("EventTarget")?;
                self.work(search(self.environments[0].bindings.len(), 11))?;
                let constructor = self.environments[0].bindings["EventTarget"].value.clone();
                let tag = self.dom_proto_text("EventTarget")?;
                self.dom_proto_property(
                    prototype,
                    tag_key.clone(),
                    Property::data(Value::String(tag), false, false, true),
                )?;
                // Each interface prototype owns its Web IDL class string;
                // AbortSignal must not inherit EventTarget's newly exposed tag.
                let signal_prototype = self.dom_proto_id("AbortSignal")?;
                let signal_tag = self.dom_proto_text("AbortSignal")?;
                self.dom_proto_property(
                    signal_prototype,
                    tag_key.clone(),
                    Property::data(Value::String(signal_tag), false, false, true),
                )?;
                records.push(InterfaceRecord {
                    prototype,
                    constructor,
                    order: 0,
                });
                continue;
            }
            self.work(4)?;
            if interface.parent.is_none() {
                return Err(ScriptError::resource("DOM interface parent missing"));
            }
            let parent_index = interfaces::parent_index(interface_index)
                .ok_or_else(|| ScriptError::resource("DOM interface parent index missing"))?;
            let parent = records.get(parent_index).ok_or_else(|| {
                ScriptError::resource("DOM interface inventory is not parent-first")
            })?;
            let parent_prototype = parent.prototype;
            let parent_constructor = parent.constructor.clone();
            let constants = if interface.name == "Node" {
                NODE_CONSTANTS
            } else {
                &[]
            };
            let unscopables = match interface.name {
                "Document" | "DocumentFragment" => PARENT_UNSCOPABLES,
                "Element" => ELEMENT_UNSCOPABLES,
                "CharacterData" | "DocumentType" => CHILD_UNSCOPABLES,
                _ => &[],
            };
            let methods = usize::from(matches!(
                interface.name,
                "Document" | "DocumentFragment" | "Element"
            )) * 3;
            let prototype = self.dom_proto_object(
                Some(Value::Object(parent_prototype)),
                2 + constants.len()
                    + methods
                    + usize::from(!unscopables.is_empty())
                    + match interface.name {
                        "Document" => 2,
                        "ProcessingInstruction" => 1,
                        "CharacterData" => 7,
                        "Node" => 10,
                        "Text" => 2,
                        _ => 0,
                    },
            )?;
            let properties =
                self.dom_proto_object(Some(parent_constructor), 3 + constants.len())?;
            let constructor = self.dom_interface_value(interface.name, properties)?;
            let name = self.dom_proto_text(interface.name)?;
            self.dom_proto_property(
                properties,
                keys[1].clone(),
                Property::data(Value::Number(interface.length as f64), false, false, true),
            )?;
            self.dom_proto_property(
                properties,
                keys[0].clone(),
                Property::data(Value::String(name.clone()), false, false, true),
            )?;
            self.dom_proto_property(
                properties,
                keys[2].clone(),
                Property::data(Value::Object(prototype), false, false, false),
            )?;
            self.dom_proto_property(
                prototype,
                tag_key.clone(),
                Property::data(Value::String(name), false, false, true),
            )?;
            if !unscopables.is_empty() {
                let bag = self.dom_proto_object(None, unscopables.len())?;
                for key in unscopables {
                    self.dom_proto_named(
                        bag,
                        key,
                        Property::data(Value::Bool(true), true, true, true),
                    )?;
                }
                self.dom_proto_property(
                    prototype,
                    unscopables_key.clone(),
                    Property::data(Value::Object(bag), false, false, true),
                )?;
            }
            if interface.name == "Document" {
                self.install_document_title(prototype)?;
            }
            if methods != 0 {
                self.install_dom_parent_prototype(interface.name, prototype)?;
            }
            if interface.name == "Node" {
                self.install_node_connected(prototype)?;
                self.install_node_data_members(prototype)?;
                self.install_node_root(prototype)?;
                self.install_node_has_child_nodes(prototype)?;
                self.install_node_normalize(prototype)?;
                self.install_node_equality(prototype)?;
                self.install_node_identity_members(prototype)?;
                self.install_node_constants(prototype, properties, &tag_key)?;
            }
            self.install_pi_members(interface.name, prototype)?;
            // Web IDL places operations/constants before this string property.
            self.dom_proto_property(
                prototype,
                keys[3].clone(),
                Property::data(constructor.clone(), true, false, true),
            )?;
            let order = first_order
                .checked_add(new_count as u64)
                .ok_or_else(|| ScriptError::resource("DOM interface order overflow"))?;
            records.push(InterfaceRecord {
                prototype,
                constructor,
                order,
            });
            new_count += 1;
        }
        let alias_order = first_order
            .checked_add(new_count as u64)
            .ok_or_else(|| ScriptError::resource("DOM alias order overflow"))?;
        let next_order = alias_order
            .checked_add(1)
            .ok_or_else(|| ScriptError::resource("DOM interface order overflow"))?;
        self.work(interfaces::interface_lookup_work(8))?;
        let document = records[interfaces::interface_index("Document").unwrap()]
            .constructor
            .clone();
        let mut global_entries = self.dom_proto_vector::<(String, Binding)>(new_count + 1)?;
        let mut alias_pending = true;
        self.work(4 * count)?;
        for index in interfaces::interface_order() {
            let interface = &INTERFACES[index];
            if interface.name == "EventTarget" {
                continue;
            }
            let record = &records[index];
            if alias_pending && interface.name > "HTMLDocument" {
                let name = self.dom_proto_owned_name("HTMLDocument")?;
                global_entries.push((name, interface_binding(document.clone(), alias_order)));
                alias_pending = false;
            }
            let name = self.dom_proto_owned_name(interface.name)?;
            global_entries.push((
                name,
                interface_binding(record.constructor.clone(), record.order),
            ));
        }
        if alias_pending {
            global_entries.push((
                self.dom_proto_owned_name("HTMLDocument")?,
                interface_binding(document, alias_order),
            ));
        }
        // Only globals need a dynamic name map. The private record vector and
        // Native's direct bag handle avoid two redundant registry rebuilds.
        let scan_work = self.environments[0]
            .bindings
            .len()
            .checked_add(global_entries.len())
            .and_then(|count| count.checked_mul(8))
            .and_then(|work| work.checked_add(64))
            .ok_or_else(|| ScriptError::resource("DOM metadata length scan overflow"))?;
        self.work(scan_work)?;
        let plan = bulk_plan(&self.environments[0].bindings, &global_entries)?;
        self.work(plan.work)?;
        self.charge(plan.bytes)?;
        self.charge((new_count + 1) * BINDING_BYTES)?;
        validate_staged(&global_entries)?;
        let global_buffer =
            self.dom_proto_vector(self.environments[0].bindings.len() + global_entries.len())?;
        // All admission precedes taking the old map. A collision is a terminal
        // initializer invariant failure: discard this private partial Runtime.
        let globals = merged_map(
            std::mem::take(&mut self.environments[0].bindings),
            global_entries,
            global_buffer,
        )?;
        self.environments[0].bindings = globals;
        self.dom_prototypes.records = records;
        self.next_global_order = next_order;
        Ok(())
    }
    fn dom_proto_vector<T>(&mut self, count: usize) -> Result<Vec<T>> {
        self.work(1)?;
        self.charge(
            count
                .checked_mul(std::mem::size_of::<T>())
                .ok_or_else(|| ScriptError::resource("DOM metadata vector storage overflow"))?,
        )?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("DOM metadata vector allocation failed"))?;
        Ok(result)
    }
    fn dom_proto_owned_name(&mut self, name: &str) -> Result<String> {
        self.work(1 + name.len())?;
        self.charge(name.len())?;
        let mut result = String::new();
        result
            .try_reserve_exact(name.len())
            .map_err(|_| ScriptError::resource("DOM metadata name allocation failed"))?;
        result.push_str(name);
        Ok(result)
    }
    pub(super) fn dom_interface_exists(&mut self, name: &str) -> Result<bool> {
        Ok(self.dom_interface_kind(name)?.is_some())
    }
    fn dom_interface_kind(&mut self, name: &str) -> Result<Option<ConstructorKind>> {
        let Some(name) = name.strip_prefix(PREFIX) else {
            return Ok(None);
        };
        self.work(interfaces::interface_lookup_work(name.len()))?;
        Ok(interfaces::interface(name).map(|interface| interface.constructor))
    }
    pub(super) fn prototype_of_in(
        &mut self,
        value: &Value,
        doc: &Document,
    ) -> Result<Option<Value>> {
        let id = match value {
            Value::Document => doc.root,
            Value::Node(id) => *id,
            _ => return Ok(self.prototype_of(value)),
        };
        self.tick()?;
        if id >= doc.nodes.len() {
            return Err(ScriptError::type_error("invalid DOM node"));
        }
        self.work(search(self.dom_prototypes.overrides.len(), 0))?;
        if let Some(value) = self.dom_prototypes.overrides.get(&id) {
            return Ok(Some(value.clone()));
        }
        let work = interfaces::mapping_work(doc, id)
            .ok_or_else(|| ScriptError::type_error("invalid DOM node"))?;
        self.work(work)?;
        let index = interfaces::node_interface_index(doc, id)
            .ok_or_else(|| ScriptError::type_error("invalid DOM node"))?;
        self.work(2)?;
        let prototype = self
            .dom_prototypes
            .records
            .get(index)
            .ok_or_else(|| ScriptError::resource("DOM prototype index missing"))?
            .prototype;
        Ok(Some(Value::Object(prototype)))
    }
    pub(super) fn dom_constructor_override(
        &mut self,
        interface: &str,
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Option<Value>> {
        let default = Value::Object(self.dom_proto_id(interface)?);
        let selected = self.get(new_target, "prototype", doc)?;
        self.tick()?;
        Ok((js_object(&selected) && selected != default).then_some(selected))
    }
    pub(super) fn dom_admit_override(&mut self, prototype: &Option<Value>) -> Result<()> {
        if prototype.is_some() {
            let count = self.dom_prototypes.overrides.len();
            self.work(search(count, 0) + moves(count))?;
            self.charge(insert_bytes::<NodeId, Value>(count))?;
        }
        Ok(())
    }
    pub(super) fn dom_publish_override(&mut self, id: NodeId, prototype: Option<Value>) {
        if let Some(prototype) = prototype {
            self.dom_prototypes.overrides.insert(id, prototype);
        }
    }
    pub(super) fn dom_interface_construct(
        &mut self,
        name: &str,
        args: &[Value],
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        let kind = self
            .dom_interface_kind(name)?
            .ok_or_else(|| ScriptError::type_error("unknown DOM interface"))?;
        match kind {
            ConstructorKind::Illegal | ConstructorKind::Html => {
                return Err(ScriptError::type_error(
                    "illegal DOM interface construction",
                ));
            }
            ConstructorKind::Document => {
                return Err(ScriptError::unsupported(
                    "independent Document construction is not implemented",
                ));
            }
            ConstructorKind::ProcessingInstruction => {
                return self.pi_construct(args, new_target, doc);
            }
            ConstructorKind::EventTarget => {
                return Err(ScriptError::type_error(
                    "invalid EventTarget constructor route",
                ));
            }
            _ => {}
        }
        let text = if matches!(kind, ConstructorKind::Text | ConstructorKind::Comment) {
            Some(match args.first() {
                None | Some(Value::Undefined) => JsString::default(),
                Some(value) => self.string_hint(value.clone(), doc)?,
            })
        } else {
            None
        };
        let interface = name.strip_prefix(PREFIX).unwrap();
        let default = Value::Object(self.dom_proto_id(interface)?);
        let selected = self.get(new_target, "prototype", doc)?;
        let prototype = if js_object(&selected) {
            selected
        } else {
            default.clone()
        };
        self.tick()?;
        let overrides_default = prototype != default;
        let plan = text
            .as_ref()
            .map(|text| self.plan_dom_data(text.units().iter().copied(), text.len(), text.len()))
            .transpose()?;
        let bytes = plan.as_ref().map_or(0, |plan| plan.stored_bytes());
        // Re-read document limits after every author conversion/prototype getter.
        if plan.is_some() {
            self.work(8)?;
        }
        if !doc.admits_text_node(bytes) {
            return Err(ScriptError::resource(
                "DOM constructor node or text limit exceeded",
            ));
        }
        self.ensure_dom_capacity(doc, 1)?;
        let count = self.dom_prototypes.overrides.len();
        // Exact character data owns one payload; its builder pays that buffer.
        // Retain the empty fragment path's existing fixed work allowance.
        if plan.is_none() {
            self.work(1)?;
        }
        if overrides_default {
            self.work(search(count, 0) + moves(count))?;
            self.charge(insert_bytes::<NodeId, Value>(count))?;
        }
        let text = match plan {
            Some(plan) => {
                let text = self.emit_dom_data(plan)?;
                self.work(8)?;
                text
            }
            None => crate::dom::DomString::default(),
        };
        self.dom_reserve_node_growth(doc, 1)?;
        let id = match kind {
            ConstructorKind::DocumentFragment => doc.create_document_fragment(),
            ConstructorKind::Text => doc
                .create_text_node_owned(text)
                .map_err(processing_instruction::dom_data_error)?,
            ConstructorKind::Comment => doc
                .create_comment_owned(text)
                .map_err(processing_instruction::dom_data_error)?,
            _ => unreachable!(),
        };
        if overrides_default {
            self.dom_prototypes.overrides.insert(id, prototype);
        }
        Ok(Value::Node(id))
    }
}
