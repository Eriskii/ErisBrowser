//! Streaming Array.from over observable synchronous iterators or array-like reads.
use super::*;

#[cfg(test)]
mod tests;

const MAX_LENGTH: u64 = 9_007_199_254_740_991;

fn lookup_work(entries: usize, units: usize) -> usize {
    let levels = entries.checked_ilog2().map_or(0, |n| n as usize + 1);
    (1 + units / 8).saturating_mul(4 * levels)
}

fn terminal(error: &ScriptError) -> bool {
    error.is_resource_limit() || error.is_unsupported()
}

impl Runtime {
    pub(super) fn install_array_from(&mut self) -> Result<()> {
        // Literal strings, registry entry and constructor property; the
        // intrinsic helper separately charges its ordinary metadata object.
        self.work(32)?;
        self.charge(1280)?;
        let function = self.intrinsic_function("Array.from", "from", 1)?;
        self.objects[self.native_properties["Array"]].insert_hidden("from".into(), function);
        Ok(())
    }

    pub(super) fn array_from(
        &mut self,
        constructor: Value,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let mapper = arguments.get(1).cloned().unwrap_or(Value::Undefined);
        // No property keys, source boxing, ignored argument scans or callbacks
        // precede this validation.
        if mapper != Value::Undefined && !json_callable(&mapper) {
            return Err(ScriptError::type_error("Array.from mapper is not callable"));
        }
        let source = arguments.first().cloned().unwrap_or(Value::Undefined);
        let this_arg = arguments.get(2).cloned().unwrap_or(Value::Undefined);
        let method = self.array_from_iterator_method(&source, doc)?;
        if let Some(method) = method {
            let result = self.array_from_result(constructor, None, doc)?;
            let iterator = self.call(method, Vec::new(), source, doc)?;
            if !js_object(&iterator) {
                return Err(ScriptError::type_error("iterator must be an object"));
            }
            let next = self.array_from_get(&iterator, "next", doc)?;
            let mut index = 0;
            loop {
                self.tick()?;
                self.array_from_count(&iterator, index, doc)?;
                // The specification forms this key before IteratorStepValue.
                let key = self.reduce_index_key(index)?;
                let step = self.call(next.clone(), Vec::new(), iterator.clone(), doc)?;
                if !js_object(&step) {
                    return Err(ScriptError::type_error("iterator result must be an object"));
                }
                if self.array_from_get(&step, "done", doc)?.truthy() {
                    self.array_from_set_length(&result, index, doc)?;
                    return Ok(result);
                }
                // Acquisition and all step/done/value errors do not close.
                let value = self.array_from_get(&step, "value", doc)?;
                let value = match self.array_from_map(&mapper, &this_arg, value, index, doc) {
                    Ok(value) => value,
                    Err(error) => return Err(self.array_from_close(&iterator, error, doc)),
                };
                if let Err(error) = self.array_from_define(&result, key, value, doc) {
                    return Err(self.array_from_close(&iterator, error, doc));
                }
                index += 1;
            }
        }
        let object = self.coerce_object(source)?;
        let length = self.array_from_get(&object, "length", doc)?;
        let length = integer_or_infinity(self.number_value(length, doc)?)
            .clamp(0.0, MAX_LENGTH as f64) as u64;
        let result = self.array_from_result(constructor, Some(length), doc)?;
        for index in 0..length {
            self.tick()?;
            let key = self.reduce_index_key(index)?;
            let value = self.reduce_get(&object, &key, doc)?;
            let value = self.array_from_map(&mapper, &this_arg, value, index, doc)?;
            self.array_from_define(&result, key, value, doc)?;
        }
        self.array_from_set_length(&result, length, doc)?;
        Ok(result)
    }

    fn array_from_result(
        &mut self,
        constructor: Value,
        length: Option<u64>,
        doc: &mut Document,
    ) -> Result<Value> {
        if self.is_constructor(constructor.clone())? {
            let arguments = match length {
                None => Vec::new(),
                Some(length) => self.array_from_arguments([Value::Number(length as f64)])?,
            };
            return self.construct(constructor, arguments, doc);
        }
        let length = length.unwrap_or(0);
        // ArrayCreate rejects the range before allocating any Array storage.
        if length > u64::from(u32::MAX) {
            return Err(ScriptError::range_error("invalid array length"));
        }
        let array = self.array(Vec::new())?;
        let Value::Array(id) = array else {
            unreachable!()
        };
        // This newly created object has not escaped. Logical length does not
        // allocate holes and uses the realm's saved intrinsic Array prototype.
        self.array_lengths[id].value = length as u32;
        Ok(array)
    }

    fn array_from_arguments<const N: usize>(&mut self, values: [Value; N]) -> Result<Vec<Value>> {
        self.work(1 + N)?;
        let bytes = N
            .checked_mul(std::mem::size_of::<Value>())
            .ok_or_else(|| ScriptError::resource("Array.from argument allocation overflow"))?;
        self.charge(bytes)?;
        let mut arguments = Vec::new();
        arguments
            .try_reserve_exact(N)
            .map_err(|_| ScriptError::resource("Array.from argument allocation failed"))?;
        arguments.extend(values);
        Ok(arguments)
    }

    fn array_from_map(
        &mut self,
        mapper: &Value,
        receiver: &Value,
        value: Value,
        index: u64,
        doc: &mut Document,
    ) -> Result<Value> {
        if *mapper == Value::Undefined {
            return Ok(value);
        }
        let arguments = self.array_from_arguments([value, Value::Number(index as f64)])?;
        self.call(mapper.clone(), arguments, receiver.clone(), doc)
    }

    fn array_from_get(
        &mut self,
        receiver: &Value,
        name: &str,
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(1 + name.len())?;
        self.charge(64)?;
        self.reduce_get(receiver, &JsString::from(name), doc)
    }

    fn array_from_native_lookup(&mut self, value: &Value, passes: usize) -> Result<()> {
        let name = match value {
            Value::Native(native) => Some(native.name.as_str()),
            Value::Json => Some("JSON"),
            Value::Math => Some("Math"),
            _ => None,
        };
        if let Some(name) = name {
            self.work(
                lookup_work(self.native_properties.len(), name.len()).saturating_mul(passes),
            )?;
        }
        Ok(())
    }

    fn array_from_iterator_method(
        &mut self,
        source: &Value,
        doc: &mut Document,
    ) -> Result<Option<Value>> {
        if matches!(source, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "Array.from items are null or undefined",
            ));
        }
        // The fixed well-known-symbol table has fifteen short literal names;
        // cloning the selected identity does not allocate.
        self.work(64)?;
        let key = self.well_known_key("iterator");
        let mut cursor = Some(source.clone());
        for _ in 0..MAX_DEPTH {
            let Some(value) = cursor else { return Ok(None) };
            self.tick()?;
            self.array_from_native_lookup(&value, 3)?;
            self.work(lookup_work(self.host_symbol_objects.len(), 1))?;
            if let Some(id) = self.symbol_property_object(&value) {
                self.work(lookup_work(self.objects[id].values.len(), 1))?;
                if let Some(property) = self.objects[id].values.get(&key).cloned() {
                    let method = match property.value {
                        PropertyValue::Data { value, .. } => value,
                        PropertyValue::Accessor {
                            get: Value::Undefined,
                            ..
                        } => Value::Undefined,
                        PropertyValue::Accessor { get, .. } => {
                            self.call(get, Vec::new(), source.clone(), doc)?
                        }
                    };
                    return match method {
                        Value::Null | Value::Undefined => Ok(None),
                        _ if json_callable(&method) => Ok(Some(method)),
                        _ => Err(ScriptError::type_error("iterator method is not callable")),
                    };
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
                    "Array.from host prototype lookup is not implemented",
                ));
            }
            if json_primitive(&value) {
                // Primitive prototype names are fixed short literals. Do not
                // box: strict getters/methods must see the original primitive.
                self.work(lookup_work(self.prototypes.len(), 8))?;
            }
            cursor = self.prototype_of(&value);
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }

    fn array_from_own_budget(
        &mut self,
        receiver: &Value,
        key: &JsString,
        passes: usize,
    ) -> Result<()> {
        self.array_from_native_lookup(receiver, passes.saturating_mul(3))?;
        let object = self.property_object(receiver).ok_or_else(|| {
            ScriptError::unsupported("Array.from host output properties are not implemented")
        })?;
        let properties = &self.objects[object];
        let work = lookup_work(properties.values.len(), key.len())
            .saturating_mul(passes)
            .saturating_add(
                lookup_work(properties.parameter_map.len(), key.len()).saturating_mul(passes + 1),
            )
            .saturating_add(match receiver {
                Value::Array(id) => {
                    lookup_work(self.array_holes[*id].len(), 1).saturating_mul(passes)
                }
                _ => 0,
            });
        self.work(1 + work)?;
        if let Some((env, name)) = self.objects[object].parameter_map.get(key) {
            // A short numeric key can name an arbitrarily long retained
            // formal. Charge before any descriptor snapshot or binding write.
            let levels = 1 + self.environments[*env]
                .bindings
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize;
            self.work(
                (1 + name.len())
                    .saturating_mul(4 * levels)
                    .saturating_mul(passes),
            )?;
        }
        Ok(())
    }

    fn array_from_define(
        &mut self,
        result: &Value,
        key: JsString,
        value: Value,
        doc: &mut Document,
    ) -> Result<()> {
        // Eight bounds the existing definition path's property/mapping/hole
        // searches and insertion. No author code occurs between this charge
        // and the numeric own definition, including mapped arguments updates.
        self.array_from_own_budget(result, &key, 8)?;
        // Fixed length-key comparisons and virtual boxed-string snapshots in
        // shared definition helpers; retained records charge separately.
        self.charge(256)?;
        if self.define_property_key(
            result,
            &PropertyKey::String(key),
            PropertyDescriptor::data_property(value, true, true, true),
            doc,
        )? {
            Ok(())
        } else {
            Err(ScriptError::type_error(
                "Array.from result property cannot be defined",
            ))
        }
    }

    fn array_from_set_length(
        &mut self,
        result: &Value,
        length: u64,
        doc: &mut Document,
    ) -> Result<()> {
        self.work(7)?;
        self.charge(64)?;
        let key = JsString::from("length");
        let mut cursor = Some(result.clone());
        let mut finished = false;
        for _ in 0..MAX_DEPTH {
            let Some(value) = cursor else {
                finished = true;
                break;
            };
            // This non-calling preflight pays for its own descriptor lookup
            // and the subsequent strict Set traversal. No author code can
            // change that chain before Set reaches its first accessor.
            self.array_from_own_budget(&value, &key, 2)?;
            self.charge(256)?;
            if self.own_property(&value, &key).is_some() {
                finished = true;
                break;
            }
            cursor = self.prototype_of(&value);
        }
        if !finished {
            return Err(ScriptError::resource("prototype chain limit exceeded"));
        }
        // Covers the write branch's additional table/mapping searches and
        // shared setter argument/key scratch before that path can allocate.
        self.array_from_own_budget(result, &key, 8)?;
        self.charge(256 + std::mem::size_of::<Value>())?;
        self.array_from_shrink_budget(result, length)?;
        self.set_key_strict(
            result.clone(),
            &key,
            Value::Number(length as f64),
            true,
            doc,
        )
    }

    fn array_from_shrink_budget(&mut self, result: &Value, length: u64) -> Result<()> {
        let Value::Array(id) = result else {
            return Ok(());
        };
        if !self.array_lengths[*id].writable || length >= u64::from(self.array_lengths[*id].value) {
            return Ok(());
        }
        let dense = self.arrays[*id].len();
        let properties = self.array_properties[*id];
        let order = &self.objects[properties].order;
        // Pay before scanning metadata or testing canonical numeric keys.
        self.work(1 + order.len().saturating_mul(12))?;
        let mut deletions = dense.saturating_sub(length.min(dense as u64) as usize);
        for key in &self.objects[properties].order {
            if key.as_string().is_some_and(|key| {
                json_array_index(key).is_some_and(|index| u64::from(index) >= length)
            }) {
                deletions = deletions.saturating_add(1);
            }
        }
        // The shared own_keys path now owns its enumeration/sort/storage
        // charges and performs no full-name deduplication. Keep this distinct
        // allowance for numeric deletion's subsequent order-vector retention.
        // A numeric deletion key compares at most ten units of another name.
        self.work(
            deletions
                .saturating_mul(self.objects[properties].order.len())
                .saturating_mul(2),
        )
    }

    fn array_from_count(&mut self, iterator: &Value, index: u64, doc: &mut Document) -> Result<()> {
        if index >= MAX_LENGTH {
            let error = ScriptError::type_error("Array.from exceeds the safe integer length");
            return Err(self.array_from_close(iterator, error, doc));
        }
        Ok(())
    }

    fn array_from_close(
        &mut self,
        iterator: &Value,
        pending: ScriptError,
        doc: &mut Document,
    ) -> ScriptError {
        if terminal(&pending) {
            return pending;
        }
        let close = (|| {
            let method = self.array_from_get(iterator, "return", doc)?;
            if matches!(method, Value::Null | Value::Undefined) {
                return Ok(Value::Undefined);
            }
            if !json_callable(&method) {
                return Err(ScriptError::type_error("iterator return is not callable"));
            }
            self.call(method, Vec::new(), iterator.clone(), doc)
        })();
        // This helper only closes pending throws. Their original identity
        // wins over every ordinary close error, including a primitive result.
        // Host terminal stops always win and do not run further author code.
        match close {
            Err(error) if terminal(&error) => error,
            _ => pending,
        }
    }
}
