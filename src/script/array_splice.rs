//! Streaming splice with live property operations and same-realm species.
use super::*;

#[cfg(test)]
mod tests;

const MAX_LENGTH: u64 = 9_007_199_254_740_991;

// Rust 1.88/1.98 B-trees use B=6. A height-h tree needs at least
// 2*6^h-1 entries; each visited node compares at most eleven keys.
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
    // Use the VM's eight-unit string-comparison work convention. Symbol and
    // integer comparisons pass zero units and require no text traversal.
    tree_bound(count).0.saturating_mul(1 + units / 8)
}

fn structural_work(count: usize) -> usize {
    // An insertion/removal can move keys, values and child handles in a node,
    // its sibling and its parent at every level (11/11/12 slots per node).
    // Include a possible new root. Existing-value replacement moves no slots.
    if count == 0 {
        0
    } else {
        tree_bound(count).1.saturating_add(1).saturating_mul(128)
    }
}

impl Runtime {
    pub(super) fn install_array_splice(&mut self) -> Result<()> {
        self.work(
            64 + search_work(self.native_properties.len(), 12).saturating_mul(2)
                + structural_work(self.native_properties.len())
                + search_work(self.prototypes.len(), 5),
        )?;
        self.charge(1280)?;
        let function = self.intrinsic_function("Array.splice", "splice", 2)?;
        let object = self.prototypes["Array"];
        self.work(
            search_work(self.objects[object].values.len(), 6).saturating_mul(2)
                + structural_work(self.objects[object].values.len()),
        )?;
        self.objects[object].insert_hidden("splice".into(), function);
        Ok(())
    }

    pub(super) fn array_splice(
        &mut self,
        receiver: Value,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        if !js_object(&receiver) && !matches!(receiver, Value::Null | Value::Undefined) {
            self.work(search_work(self.prototypes.len(), 7).saturating_mul(2))?;
        }
        let object = self.coerce_object(receiver)?;
        let length = self.splice_named_get(&object, "length", doc)?;
        let length = integer_or_infinity(self.splice_number(length, doc)?)
            .clamp(0.0, MAX_LENGTH as f64) as u64;
        let start = arguments.first().cloned().unwrap_or(Value::Undefined);
        let start = integer_or_infinity(self.splice_number(start, doc)?);
        let start = if start < 0.0 {
            (length as f64 + start).max(0.0) as u64
        } else {
            start.min(length as f64) as u64
        };
        let items = arguments.get(2..).unwrap_or(&[]);
        let item_count = items.len() as u64;
        let deleted = if arguments.is_empty() {
            0
        } else if arguments.len() == 1 {
            length - start
        } else {
            integer_or_infinity(self.splice_number(arguments[1].clone(), doc)?)
                .clamp(0.0, (length - start) as f64) as u64
        };
        let final_length = (length - deleted)
            .checked_add(item_count)
            .filter(|length| *length <= MAX_LENGTH)
            .ok_or_else(|| ScriptError::type_error("splice length exceeds safe integer range"))?;
        let result = self.splice_species(&object, deleted, doc)?;
        for index in 0..deleted {
            self.tick()?;
            let from = self.reduce_index_key(start + index)?;
            if self.splice_property(&object, &from)?.is_some() {
                let value = self.splice_get(&object, &from, doc)?;
                // Unlike movement keys, this key is formed after source Get.
                let to = self.reduce_index_key(index)?;
                self.splice_define(&result, to, value, true, doc)?;
            }
        }
        self.splice_length(&result, deleted, doc)?;
        if item_count < deleted {
            for index in start..length - deleted {
                self.tick()?;
                let from = self.reduce_index_key(index + deleted)?;
                let to = self.reduce_index_key(index + item_count)?;
                if self.splice_property(&object, &from)?.is_some() {
                    let value = self.splice_get(&object, &from, doc)?;
                    self.splice_set(&object, to, value, doc)?;
                } else {
                    self.splice_delete(&object, &to)?;
                }
            }
            for index in (final_length..length).rev() {
                self.tick()?;
                let key = self.reduce_index_key(index)?;
                self.splice_delete(&object, &key)?;
            }
        } else if item_count > deleted {
            for index in (start..length - deleted).rev() {
                self.tick()?;
                let from = self.reduce_index_key(index + deleted)?;
                let to = self.reduce_index_key(index + item_count)?;
                if self.splice_property(&object, &from)?.is_some() {
                    let value = self.splice_get(&object, &from, doc)?;
                    self.splice_set(&object, to, value, doc)?;
                } else {
                    self.splice_delete(&object, &to)?;
                }
            }
        }
        for (offset, value) in items.iter().enumerate() {
            self.tick()?;
            let key = self.reduce_index_key(start + offset as u64)?;
            self.splice_set(&object, key, value.clone(), doc)?;
        }
        self.splice_length(&object, final_length, doc)?;
        Ok(result)
    }

    fn splice_native_budget(&mut self, value: &Value, passes: usize) -> Result<()> {
        let name = match value {
            Value::Native(native) => Some(native.name.as_str()),
            Value::Json => Some("JSON"),
            Value::Math => Some("Math"),
            _ => None,
        };
        if let Some(name) = name {
            self.work(
                search_work(self.native_properties.len(), name.len()).saturating_mul(passes),
            )?;
        }
        Ok(())
    }

    fn splice_own_budget(
        &mut self,
        object: &Value,
        key: &JsString,
        passes: usize,
    ) -> Result<usize> {
        // property_object here, in own_property/definition and prototype_of.
        self.splice_native_budget(object, passes.saturating_mul(3))?;
        let id = self.property_object(object).ok_or_else(|| {
            ScriptError::unsupported("splice host property operation is not implemented")
        })?;
        let properties = &self.objects[id];
        let work = search_work(properties.values.len(), key.len())
            .saturating_mul(passes)
            .saturating_add(
                search_work(properties.parameter_map.len(), key.len())
                    .saturating_mul(passes.saturating_add(1)),
            )
            .saturating_add(match object {
                Value::Array(array) => {
                    search_work(self.array_holes[*array].len(), 0).saturating_mul(passes)
                }
                _ => 0,
            });
        self.work(1 + work)?;
        if let Some((env, name)) = self.objects[id].parameter_map.get(key) {
            // Borrow the formal name; never clone a long mapping before its
            // environment comparison work has been paid.
            self.work(
                search_work(self.environments[*env].bindings.len(), name.len())
                    .saturating_mul(passes),
            )?;
        }
        Ok(id)
    }

    fn splice_own(&mut self, object: &Value, key: &JsString) -> Result<Option<Property>> {
        let id = self.splice_own_budget(object, key, 1)?;
        // A boxed-string index snapshot creates a one-code-unit String.
        if matches!(self.objects[id].boxed, Some(Value::String(_))) {
            self.charge(128)?;
        }
        self.work(2 + key.len())?;
        Ok(self.own_property(object, key))
    }

    pub(super) fn splice_property(
        &mut self,
        object: &Value,
        key: &JsString,
    ) -> Result<Option<Property>> {
        let mut cursor = Some(object.clone());
        for _ in 0..MAX_DEPTH {
            let Some(value) = cursor else { return Ok(None) };
            self.tick()?;
            if let Some(property) = self.splice_own(&value, key)? {
                return Ok(Some(property));
            }
            cursor = self.prototype_of(&value);
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }

    pub(super) fn splice_get(
        &mut self,
        object: &Value,
        key: &JsString,
        doc: &mut Document,
    ) -> Result<Value> {
        let Some(property) = self.splice_property(object, key)? else {
            return Ok(Value::Undefined);
        };
        self.splice_read_value(object, property, doc)
    }

    fn splice_read_value(
        &mut self,
        object: &Value,
        property: Property,
        doc: &mut Document,
    ) -> Result<Value> {
        match property.value {
            PropertyValue::Data { value, .. } => Ok(value),
            PropertyValue::Accessor {
                get: Value::Undefined,
                ..
            } => Ok(Value::Undefined),
            PropertyValue::Accessor { get, .. } => self.call(get, Vec::new(), object.clone(), doc),
        }
    }

    fn splice_key(&mut self, name: &str) -> Result<JsString> {
        self.work(1 + 3 * name.len())?;
        self.charge(64 + name.len() * 6)?;
        Ok(name.into())
    }

    pub(super) fn splice_named_get(
        &mut self,
        object: &Value,
        name: &str,
        doc: &mut Document,
    ) -> Result<Value> {
        let key = self.splice_key(name)?;
        self.splice_get(object, &key, doc)
    }

    pub(super) fn splice_symbol_get(
        &mut self,
        object: &Value,
        name: &str,
        doc: &mut Document,
    ) -> Result<Value> {
        // symbols::WELL_KNOWN is the fixed fifteen-entry intrinsic table;
        // author-created/registered symbols cannot grow it.
        self.work(search_work(15, name.len()))?;
        let key = self.well_known_key(name);
        let mut cursor = Some(object.clone());
        for _ in 0..MAX_DEPTH {
            let Some(value) = cursor else {
                return Ok(Value::Undefined);
            };
            self.tick()?;
            self.splice_native_budget(&value, 2)?;
            self.work(search_work(self.host_symbol_objects.len(), 0))?;
            if let Some(id) = self.symbol_property_object(&value) {
                self.work(search_work(self.objects[id].values.len(), 0))?;
                if let Some(property) = self.objects[id].values.get(&key).cloned() {
                    return self.splice_read_value(object, property, doc);
                }
            }
            if matches!(
                value,
                Value::Window
                    | Value::Document
                    | Value::Node(_)
                    | Value::Console
                    | Value::Style(_)
                    | Value::ClassList(_)
            ) {
                return Err(ScriptError::unsupported(
                    "splice host symbol lookup is not implemented",
                ));
            }
            cursor = self.prototype_of(&value);
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }

    fn splice_argument(&mut self, value: Value) -> Result<Vec<Value>> {
        self.tick()?;
        self.charge(std::mem::size_of::<Value>())?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(1)
            .map_err(|_| ScriptError::resource("splice argument allocation failed"))?;
        values.push(value);
        Ok(values)
    }

    pub(super) fn splice_number(&mut self, value: Value, doc: &mut Document) -> Result<f64> {
        if !js_object(&value) {
            return self.primitive_number_value(value);
        }
        let method = self.splice_symbol_get(&value, "toPrimitive", doc)?;
        if !matches!(method, Value::Undefined | Value::Null) {
            if !json_callable(&method) {
                return Err(ScriptError::type_error(
                    "Symbol.toPrimitive must be callable",
                ));
            }
            let hint = self.splice_key("number")?;
            let arguments = self.splice_argument(Value::String(hint))?;
            let primitive = self.call(method, arguments, value.clone(), doc)?;
            if js_object(&primitive) {
                return Err(ScriptError::type_error(
                    "Symbol.toPrimitive must return a primitive",
                ));
            }
            return self.primitive_number_value(primitive);
        }
        for name in ["valueOf", "toString"] {
            let method = self.splice_named_get(&value, name, doc)?;
            if json_callable(&method) {
                let primitive = self.call(method, Vec::new(), value.clone(), doc)?;
                if !js_object(&primitive) {
                    return self.primitive_number_value(primitive);
                }
            }
        }
        Err(ScriptError::type_error(
            "object cannot be converted to a number",
        ))
    }

    pub(super) fn splice_species(
        &mut self,
        object: &Value,
        length: u64,
        doc: &mut Document,
    ) -> Result<Value> {
        if matches!(object, Value::Array(_)) {
            let mut constructor = self.splice_named_get(object, "constructor", doc)?;
            // All represented functions belong to this realm. Retain the
            // IsConstructor step, including bound-function traversal; there
            // is no cross-realm intrinsic substitution to implement here.
            self.is_constructor(constructor.clone())?;
            if js_object(&constructor) {
                constructor = self.splice_symbol_get(&constructor, "species", doc)?;
                if constructor == Value::Null {
                    constructor = Value::Undefined;
                }
            }
            if constructor != Value::Undefined {
                if !self.is_constructor(constructor.clone())? {
                    return Err(ScriptError::type_error(
                        "array species is not a constructor",
                    ));
                }
                let arguments = self.splice_argument(Value::Number(length as f64))?;
                return self.construct(constructor, arguments, doc);
            }
        }
        self.splice_array_create(length)
    }

    fn splice_array_create(&mut self, length: u64) -> Result<Value> {
        if length > u64::from(u32::MAX) {
            return Err(ScriptError::range_error("invalid array length"));
        }
        let required =
            32 + 72 + std::mem::size_of::<Option<AbortSlot>>() + std::mem::size_of::<Option<f64>>();
        if required > MAX_HEAP.saturating_sub(self.allocated) {
            return Err(ScriptError::resource("script allocation limit exceeded"));
        }
        self.work(search_work(self.prototypes.len(), 6).saturating_mul(2))?;
        let array = self.array(Vec::new())?;
        let Value::Array(id) = array else {
            unreachable!()
        };
        self.array_lengths[id].value = length as u32;
        Ok(array)
    }

    pub(super) fn splice_define(
        &mut self,
        object: &Value,
        key: JsString,
        value: Value,
        create: bool,
        doc: &mut Document,
    ) -> Result<()> {
        // Eight values/mapping/hole searches cover the shared definition's
        // snapshot, dense-cache check, membership check and insertion.
        let id = self.splice_own_budget(object, &key, 8)?;
        self.work(32 + 4 * key.len())?;
        self.charge(256)?;
        let sidecar = self.objects[id]
            .values
            .contains_key(&PropertyKey::String(key.clone()));
        let mut stored = true;
        if let Value::Array(array) = object {
            if key.units() == [108, 101, 110, 103, 116, 104] {
                stored = false;
                if let Value::Number(length) = &value {
                    self.splice_shrink_budget(*array, *length)?;
                }
            } else if !sidecar && let Some(index) = json_array_index(&key) {
                let index = index as usize;
                let dense = &self.arrays[*array];
                if index < dense.len() {
                    stored = false;
                    self.work(structural_work(self.array_holes[*array].len()))?;
                } else if index == dense.len() && index < 65_536 {
                    stored = false;
                    if dense.len() == dense.capacity() {
                        let moved = dense.len();
                        self.work(moved)?;
                        // Vec's amortized growth uses at most max(2*len,4)
                        // slots for these non-ZST handles. Pay the whole new
                        // buffer, not just its net retained-slot increase.
                        self.charge(
                            moved
                                .saturating_mul(2)
                                .max(4)
                                .saturating_mul(std::mem::size_of::<Value>()),
                        )?;
                    }
                }
            }
        }
        if stored && !sidecar {
            self.work(structural_work(self.objects[id].values.len()).saturating_add(1))?;
            let order = &self.objects[id].order;
            if order.len() == order.capacity() {
                let moved = order.len();
                self.work(moved)?;
                self.charge(
                    moved
                        .saturating_mul(2)
                        .max(4)
                        .saturating_mul(std::mem::size_of::<PropertyKey>()),
                )?;
            }
        }
        let desc = if create {
            PropertyDescriptor::data_property(value, true, true, true)
        } else {
            PropertyDescriptor {
                value: Some(value),
                ..PropertyDescriptor::default()
            }
        };
        if self.define_property_key(object, &PropertyKey::String(key), desc, doc)? {
            Ok(())
        } else {
            Err(ScriptError::type_error("splice property cannot be defined"))
        }
    }

    fn splice_set(
        &mut self,
        object: &Value,
        key: JsString,
        value: Value,
        doc: &mut Document,
    ) -> Result<()> {
        if let Some(property) = self.splice_property(object, &key)? {
            match property.value {
                PropertyValue::Accessor {
                    set: Value::Undefined,
                    ..
                }
                | PropertyValue::Data {
                    writable: false, ..
                } => {
                    return Err(ScriptError::type_error(
                        "splice property cannot be assigned",
                    ));
                }
                PropertyValue::Accessor { set, .. } => {
                    let arguments = self.splice_argument(value)?;
                    self.call(set, arguments, object.clone(), doc)?;
                    return Ok(());
                }
                _ => {}
            }
        }
        // No author callback intervenes between the live walk and definition.
        // Shared definition preserves mapped arguments and ArraySetLength.
        let create = self.splice_own(object, &key)?.is_none();
        self.splice_define(object, key, value, create, doc)
    }

    pub(super) fn splice_length(
        &mut self,
        object: &Value,
        length: u64,
        doc: &mut Document,
    ) -> Result<()> {
        let key = self.splice_key("length")?;
        self.splice_set(object, key, Value::Number(length as f64), doc)
    }

    fn splice_delete(&mut self, object: &Value, key: &JsString) -> Result<()> {
        let Some(property) = self.splice_own(object, key)? else {
            return Ok(());
        };
        if !property.configurable {
            return Err(ScriptError::type_error("splice property cannot be deleted"));
        }
        let id = self.splice_own_budget(object, key, 4)?;
        let properties = &self.objects[id];
        let work = properties
            .order
            .len()
            .saturating_mul(2 + key.len() / 8)
            .saturating_add(structural_work(properties.values.len()))
            .saturating_add(structural_work(properties.parameter_map.len()));
        self.work(work)?;
        if let Value::Array(array) = object
            && json_array_index(key)
                .is_some_and(|index| (index as usize) < self.arrays[*array].len())
        {
            self.work(structural_work(self.array_holes[*array].len()))?;
        }
        self.charge(128)?;
        if self.delete_property(object.clone(), key)? {
            Ok(())
        } else {
            Err(ScriptError::type_error("splice property cannot be deleted"))
        }
    }

    fn splice_shrink_budget(&mut self, array: usize, length: f64) -> Result<()> {
        if !(0.0..=u32::MAX as f64).contains(&length) || length.fract() != 0.0 {
            return Ok(()); // Preserve shared ArraySetLength's RangeError.
        }
        let length = length as u32;
        if !self.array_lengths[array].writable || length >= self.array_lengths[array].value {
            return Ok(());
        }
        let id = self.array_properties[array];
        let properties = &self.objects[id];
        let order = properties.order.len();
        self.work(1 + order.saturating_mul(12))?;
        let mut deletions = self.arrays[array].len().saturating_sub(length as usize);
        for key in &self.objects[id].order {
            if key
                .as_string()
                .is_some_and(|s| json_array_index(s).is_some_and(|i| i >= length))
            {
                deletions = deletions.saturating_add(1);
            }
        }
        let properties = &self.objects[id];
        let per_delete = search_work(properties.values.len(), 10)
            .saturating_mul(3)
            .saturating_add(search_work(properties.parameter_map.len(), 10).saturating_mul(2))
            .saturating_add(
                search_work(self.array_holes[array].len().saturating_add(deletions), 0)
                    .saturating_mul(2),
            )
            .saturating_add(structural_work(properties.values.len()))
            .saturating_add(structural_work(properties.parameter_map.len()))
            .saturating_add(structural_work(
                self.array_holes[array].len().saturating_add(deletions),
            ))
            .saturating_add(order.saturating_mul(3));
        // own_keys pays its own snapshot/sort/virtual-key allocation costs.
        // These are the subsequent short-key deletions/retention operations,
        // bounded by actual retained slots/records, never logical gaps.
        self.work(deletions.saturating_mul(per_delete))
    }
}
