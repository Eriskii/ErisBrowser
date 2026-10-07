//! Non-shared views with live backing witnesses and bounded scalar byte access.
use super::array_buffer::BufferId;
use super::scalar_codec::Codec;
#[cfg(test)]
use super::scalar_codec::{decode_f16, encode_f16};
use super::*;
use std::collections::btree_map::Entry;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug)]
enum ViewLength {
    Fixed(usize),
    Tracking,
}

#[derive(Clone, Copy, Debug)]
struct Record {
    object_id: usize,
    buffer: BufferId,
    offset: usize,
    length: ViewLength,
}

#[derive(Default)]
pub(super) struct State {
    // No callback follows ordinary object creation and record publication.
    // These sorted, append-only indices and copied BufferIds survive callbacks.
    records: Vec<Record>,
    prototype: Option<usize>,
}

// Same pinned B=6 installation bound as ArrayBuffer. Fixed keys are short;
// no user text or backing bytes are traversed by this metadata accounting.
fn tree_bound(count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let mut capacity = count.saturating_add(1) / 2;
    let mut nodes = 1usize;
    while capacity >= 6 {
        capacity /= 6;
        nodes += 1;
    }
    (count.min(11usize.saturating_mul(nodes)), nodes)
}

fn search_work(count: usize, units: usize) -> usize {
    tree_bound(count).0.saturating_mul(1 + units / 8)
}

fn insertion_work(count: usize) -> usize {
    // Per possible level: <=12 fixed key/value-entry moves and <=12 logical
    // edge/backlink updates, plus four header/setup operations. These are not
    // byte or machine-word counts. Include a possible new root.
    tree_bound(count).1.saturating_add(1).saturating_mul(28)
}

fn node_bytes<K, V>() -> usize {
    16 * (std::mem::size_of::<K>() + std::mem::size_of::<V>())
        + 32 * std::mem::size_of::<usize>()
        + 64
}

fn insertion_bytes<K, V>(count: usize) -> Result<usize> {
    // These trees already contain entries. Pay the full possible split chain;
    // do not borrow amortized fresh-tree credit from earlier installations.
    tree_bound(count)
        .1
        .saturating_add(1)
        .checked_mul(node_bytes::<K, V>())
        .ok_or_else(|| ScriptError::resource("DataView intrinsic storage overflow"))
}

impl Runtime {
    fn data_view_reserve_order(&mut self, owner: usize, capacity: usize) -> Result<()> {
        let order = &self.objects[owner].order;
        if order.capacity() >= capacity {
            return self.tick();
        }
        let additional = capacity
            .checked_sub(order.len())
            .ok_or_else(|| ScriptError::resource("DataView intrinsic order overflow"))?;
        let bytes = capacity
            .checked_mul(std::mem::size_of::<PropertyKey>())
            .ok_or_else(|| ScriptError::resource("DataView intrinsic order overflow"))?;
        self.work(1 + order.len().saturating_mul(2))?;
        self.charge(bytes)?;
        self.objects[owner]
            .order
            .try_reserve_exact(additional)
            .map_err(|_| ScriptError::resource("DataView intrinsic order allocation failed"))
    }

    fn data_view_install_function(
        &mut self,
        owner: usize,
        full: &str,
        display: &str,
        length: usize,
        key: &str,
        getter: bool,
    ) -> Result<()> {
        if self.objects[owner].order.len() == self.objects[owner].order.capacity() {
            return Err(ScriptError::resource(
                "DataView intrinsic order was not reserved",
            ));
        }
        let registry = self.native_properties.len();
        let properties = self.objects[owner].values.len();
        self.work(
            64 + search_work(registry, full.len())
                + insertion_work(registry)
                + search_work(properties, key.len())
                + insertion_work(properties)
                + 1,
        )?;
        // Retain the original fixed native/string allowances and both ordinary
        // property charges. Separately pay actual-type B-tree nodes and order
        // buffers, including worst-case splits of the already populated trees.
        self.charge(
            1536 + 8 * (full.len() + display.len())
                + 552
                + node_bytes::<PropertyKey, Property>()
                + 2 * std::mem::size_of::<PropertyKey>(),
        )?;
        self.charge(insertion_bytes::<String, usize>(registry)?)?;
        self.charge(insertion_bytes::<PropertyKey, Property>(properties)?)?;
        // Keep the ordinary object base debit and publication position, but
        // construct the final prototype without an unused Object lookup.
        self.charge(
            72 + std::mem::size_of::<Option<AbortSlot>>() + std::mem::size_of::<Option<f64>>(),
        )?;
        let id = self.objects.len();
        self.objects.push(ScriptObject {
            prototype: Some(Value::Function(self.function_prototype)),
            ..ScriptObject::default()
        });
        let bag = &mut self.objects[id];
        bag.order
            .try_reserve_exact(2)
            .map_err(|_| ScriptError::resource("DataView function order allocation failed"))?;
        // Fresh two-key bag: one comparison, three entry moves, one leaf.
        // Final descriptors remove the later attributes() lookup and rewrite.
        for (key, value) in [
            (PropertyKey::from("name"), Value::String(display.into())),
            (PropertyKey::from("length"), Value::Number(length as f64)),
        ] {
            let Entry::Vacant(entry) = bag.values.entry(key.clone()) else {
                unreachable!()
            };
            bag.order.push(key);
            entry.insert(Property::data(value, false, false, true));
        }
        let function = self.alloc_native(full, Value::Undefined)?;
        let property = if getter {
            Property {
                value: PropertyValue::Accessor {
                    get: function,
                    set: Value::Undefined,
                },
                enumerable: false,
                configurable: true,
            }
        } else {
            Property::data(function, true, false, true)
        };
        let key = PropertyKey::from(key);
        let registry_key = full.to_owned();
        // All fallible admission and whole-Runtime calls precede both borrows.
        // entry performs the only search, and no insertion replaces an entry.
        let registry_entry = self.native_properties.entry(registry_key);
        let object = &mut self.objects[owner];
        let property_entry = object.values.entry(key.clone());
        let (Entry::Vacant(registry_entry), Entry::Vacant(property_entry)) =
            (registry_entry, property_entry)
        else {
            return Err(ScriptError::resource("duplicate DataView intrinsic"));
        };
        object.order.push(key);
        registry_entry.insert(id);
        property_entry.insert(property);
        Ok(())
    }

    fn data_view_install_tag(&mut self, prototype: usize) -> Result<()> {
        self.work(search_work(15, 11))?;
        let tag = self.well_known_key("toStringTag");
        let count = self.objects[prototype].values.len();
        self.work(search_work(count, 0) + insertion_work(count) + 1)?;
        self.charge(512 + insertion_bytes::<PropertyKey, Property>(count)?)?;
        let property = Property::data(Value::String("DataView".into()), false, false, true);
        let object = &mut self.objects[prototype];
        if object.order.len() == object.order.capacity() {
            return Err(ScriptError::resource(
                "DataView intrinsic order was not reserved",
            ));
        }
        let Entry::Vacant(entry) = object.values.entry(tag.clone()) else {
            return Err(ScriptError::resource("duplicate DataView intrinsic"));
        };
        object.order.push(tag);
        entry.insert(property);
        Ok(())
    }

    pub(super) fn install_data_view_intrinsics(&mut self) -> Result<()> {
        self.work(64 + search_work(self.prototypes.len(), 8))?;
        self.charge(256)?;
        let prototype = self.prototypes["DataView"];
        self.data_views.prototype = Some(prototype);
        // Constructor, three getters, nine pairs and the tag. Reserve once;
        // insertion order remains identical to the individual installer path.
        self.data_view_reserve_order(prototype, 23)?;
        for (name, full, display) in [
            ("buffer", "DataView.getBuffer", "get buffer"),
            ("byteLength", "DataView.getByteLength", "get byteLength"),
            ("byteOffset", "DataView.getByteOffset", "get byteOffset"),
        ] {
            self.work(32)?;
            self.charge(256)?;
            self.data_view_install_function(prototype, full, display, 0, name, true)?;
        }
        for suffix in Codec::NAMES {
            for (prefix, length) in [("get", 1), ("set", 2)] {
                // Both formatted names and their UTF-16/key handles are paid
                // before materialization. Names have fixed bounded lengths.
                self.work(64)?;
                self.charge(512)?;
                let name = format!("{prefix}{suffix}");
                let full = format!("DataView.{name}");
                self.data_view_install_function(prototype, &full, &name, length, &name, false)?;
            }
        }
        self.data_view_install_tag(prototype)
    }

    fn data_view_find(&mut self, value: &Value) -> Result<Option<usize>> {
        let Value::Object(id) = value else {
            return Ok(None);
        };
        let mut first = 0;
        let mut last = self.data_views.records.len();
        while first < last {
            self.tick()?;
            let middle = first + (last - first) / 2;
            match self.data_views.records[middle].object_id.cmp(id) {
                Ordering::Less => first = middle + 1,
                Ordering::Greater => last = middle,
                Ordering::Equal => return Ok(Some(middle)),
            }
        }
        Ok(None)
    }

    pub(super) fn data_view_is_view(&mut self, value: &Value) -> Result<bool> {
        // Empty state performs no search: ArrayBuffer's existing dispatch tick
        // covers this predicate, including the original cheap no-view path.
        Ok(self.data_view_find(value)?.is_some())
    }

    fn data_view_record(&mut self, value: &Value) -> Result<Record> {
        self.data_view_find(value)?
            .map(|index| self.data_views.records[index])
            .ok_or_else(|| ScriptError::type_error("receiver is not a DataView"))
    }

    fn data_view_reserve_record(&mut self) -> Result<()> {
        let records = &self.data_views.records;
        if records.len() < records.capacity() {
            return self.tick();
        }
        let capacity = records
            .capacity()
            .checked_mul(2)
            .map(|n| n.max(4))
            .ok_or_else(|| ScriptError::resource("DataView record capacity overflow"))?;
        let bytes = capacity
            .checked_mul(std::mem::size_of::<Record>())
            .ok_or_else(|| ScriptError::resource("DataView record allocation overflow"))?;
        let moved = records
            .len()
            .checked_mul(std::mem::size_of::<Record>())
            .ok_or_else(|| ScriptError::resource("DataView record movement overflow"))?;
        let additional = capacity - records.len();
        self.work(1 + moved.div_ceil(8).saturating_mul(2))?;
        // Full requested block and logical old/new overlap, not a measurement
        // of allocator rounding/overhead (reserve_exact may grant more).
        self.charge(bytes)?;
        self.data_views
            .records
            .try_reserve_exact(additional)
            .map_err(|_| ScriptError::resource("DataView record allocation failed"))
    }

    fn data_view_length(&mut self, record: Record) -> Result<usize> {
        let metadata = self.buffer_view_metadata(record.buffer)?;
        let length = metadata
            .byte_length
            .ok_or_else(|| ScriptError::type_error("DataView buffer is detached"))?;
        let remaining = length
            .checked_sub(record.offset)
            .ok_or_else(|| ScriptError::type_error("DataView is out of bounds"))?;
        match record.length {
            ViewLength::Tracking => Ok(remaining),
            ViewLength::Fixed(length) if length <= remaining => Ok(length),
            ViewLength::Fixed(_) => Err(ScriptError::type_error("DataView is out of bounds")),
        }
    }

    pub(super) fn data_view_constructor(
        &mut self,
        arguments: &[Value],
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let buffer = self.buffer_for_view(arguments.first().unwrap_or(&Value::Undefined))?;
        let offset =
            self.buffer_index(arguments.get(1).cloned().unwrap_or(Value::Undefined), doc)?;
        let metadata = self.buffer_view_metadata(buffer)?;
        let captured = metadata
            .byte_length
            .ok_or_else(|| ScriptError::type_error("DataView buffer is detached"))?
            as u64;
        if offset > captured {
            return Err(ScriptError::range_error("DataView offset exceeds buffer"));
        }
        let explicit = arguments
            .get(2)
            .filter(|value| !matches!(value, Value::Undefined));
        let length = if let Some(value) = explicit {
            let length = self.buffer_index(value.clone(), doc)?;
            if length > captured - offset {
                return Err(ScriptError::range_error("DataView length exceeds buffer"));
            }
            ViewLength::Fixed(length as usize)
        } else if metadata.resizable {
            ViewLength::Tracking
        } else {
            ViewLength::Fixed((captured - offset) as usize)
        };
        // Explicit length conversion used captured bounds. Its detach/resize
        // effects are checked only after prototype lookup and object creation.
        let prototype = self.splice_named_get(&new_target, "prototype", doc)?;
        let prototype = if js_object(&prototype) {
            prototype
        } else {
            Value::Object(self.data_views.prototype.expect("DataView initialized"))
        };
        let object = self.object_ordered([])?;
        let Value::Object(object_id) = object else {
            unreachable!()
        };
        self.objects[object_id].prototype = Some(prototype);
        let current = self
            .buffer_view_metadata(buffer)?
            .byte_length
            .ok_or_else(|| ScriptError::type_error("DataView buffer is detached"))?
            as u64;
        if offset > current {
            return Err(ScriptError::range_error("DataView offset exceeds buffer"));
        }
        if explicit.is_some()
            && let ViewLength::Fixed(length) = length
            && length as u64 > current - offset
        {
            return Err(ScriptError::range_error("DataView length exceeds buffer"));
        }
        self.data_view_reserve_record()?;
        debug_assert!(
            self.data_views
                .records
                .last()
                .is_none_or(|record| record.object_id < object_id)
        );
        self.data_views.records.push(Record {
            object_id,
            buffer,
            offset: offset as usize,
            length,
        });
        Ok(object)
    }

    pub(super) fn data_view_native(
        &mut self,
        method: &str,
        receiver: Value,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let record = self.data_view_record(&receiver)?;
        if method == "getBuffer" {
            return self.buffer_view_value(record.buffer);
        }
        if matches!(method, "getByteLength" | "getByteOffset") {
            let length = self.data_view_length(record)?;
            return Ok(Value::Number(if method == "getByteLength" {
                length
            } else {
                record.offset
            } as f64));
        }
        let (write, suffix) = if let Some(suffix) = method.strip_prefix("get") {
            (false, suffix)
        } else if let Some(suffix) = method.strip_prefix("set") {
            (true, suffix)
        } else {
            return Err(ScriptError::type_error("unknown DataView method"));
        };
        let codec = Codec::named(suffix)
            .ok_or_else(|| ScriptError::type_error("unknown DataView method"))?;
        let index =
            self.buffer_index(arguments.first().cloned().unwrap_or(Value::Undefined), doc)?;
        let value = if write {
            self.splice_number(arguments.get(1).cloned().unwrap_or(Value::Undefined), doc)?
        } else {
            0.0
        };
        let little = arguments
            .get(if write { 2 } else { 1 })
            .is_some_and(Value::truthy);
        let length = self.data_view_length(record)?;
        let width = codec.width();
        if index
            .checked_add(width as u64)
            .is_none_or(|end| end > length as u64)
        {
            return Err(ScriptError::range_error("DataView element exceeds view"));
        }
        let offset = record
            .offset
            .checked_add(index as usize)
            .ok_or_else(|| ScriptError::range_error("DataView element offset overflow"))?;
        self.work(codec.work())?;
        if write {
            let bytes = codec.encode(value, little);
            self.buffer_view_write(record.buffer, offset, &bytes[..width])?;
            Ok(Value::Undefined)
        } else {
            let bytes = self.buffer_view_read(record.buffer, offset, width)?;
            Ok(Value::Number(codec.decode(bytes, little)))
        }
    }
}
