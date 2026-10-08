//! Reflect property operations with independent target and Receiver identities.
//! Existing assignment/Object entry points keep their own conversion rules.
use super::*;

#[cfg(test)]
mod tests;

fn search(entries: usize, units: usize) -> usize {
    let levels = entries.checked_ilog2().map_or(0, |n| n as usize + 1);
    entries.min(11 * levels).saturating_mul(1 + units)
}

impl Runtime {
    pub(super) fn reflect_property_call(
        &mut self,
        name: &str,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        // Includes result assembly before any callback or eventual mutation.
        self.work(8)?;
        let target = args.first().cloned().unwrap_or(Value::Undefined);
        if !js_object(&target) {
            return Err(ScriptError::type_error("Reflect target must be an object"));
        }
        match name {
            "Reflect.getPrototypeOf" => {
                return Ok(self.reflect_prototype(&target, doc)?.unwrap_or(Value::Null));
            }
            "Reflect.setPrototypeOf" => {
                let next = args.get(1).cloned().unwrap_or(Value::Undefined);
                if next != Value::Null && !js_object(&next) {
                    return Err(ScriptError::type_error(
                        "prototype must be an object or null",
                    ));
                }
                return self
                    .reflect_set_prototype(&target, next, doc)
                    .map(Value::Bool);
            }
            "Reflect.isExtensible" | "Reflect.preventExtensions" => {
                self.reflect_registry_admission(&target, 1)?;
                let id = self.property_object(&target).ok_or_else(|| {
                    ScriptError::unsupported("host extensibility is not implemented")
                })?;
                self.work(4)?;
                if name == "Reflect.isExtensible" {
                    return Ok(Value::Bool(!self.objects[id].non_extensible));
                }
                if !self.typed_array_can_prevent_extensions(&target)? {
                    return Ok(Value::Bool(false));
                }
                self.objects[id].non_extensible = true;
                return Ok(Value::Bool(true));
            }
            _ => {}
        }
        // Target validation precedes every observable key conversion.
        let key = self.property_key(args.get(1).cloned().unwrap_or(Value::Undefined), doc)?;
        match name {
            "Reflect.has" => self
                .reflect_find(&target, &key, false, doc)
                .map(|property| Value::Bool(property.is_some())),
            "Reflect.get" => {
                let receiver = args.get(2).cloned().unwrap_or_else(|| target.clone());
                match self.reflect_find(&target, &key, true, doc)? {
                    None => Ok(Value::Undefined),
                    Some(Property {
                        value: PropertyValue::Data { value, .. },
                        ..
                    }) => Ok(value),
                    Some(Property {
                        value:
                            PropertyValue::Accessor {
                                get: Value::Undefined,
                                ..
                            },
                        ..
                    }) => Ok(Value::Undefined),
                    Some(Property {
                        value: PropertyValue::Accessor { get, .. },
                        ..
                    }) => self.call(get, Vec::new(), receiver, doc),
                }
            }
            "Reflect.set" => {
                let value = args.get(2).cloned().unwrap_or(Value::Undefined);
                let receiver = args.get(3).cloned().unwrap_or_else(|| target.clone());
                self.reflect_set(&target, &key, value, receiver, doc)
                    .map(Value::Bool)
            }
            "Reflect.deleteProperty" => self.delete_property_key(target, &key).map(Value::Bool),
            "Reflect.getOwnPropertyDescriptor" => {
                let (property, _) = self.reflect_own(&target, &key, true)?;
                match property {
                    Some(property) => self.reflect_descriptor(property),
                    None => Ok(Value::Undefined),
                }
            }
            _ => Err(ScriptError::unsupported(
                "Reflect operation is not implemented",
            )),
        }
    }

    fn reflect_registry_admission(&mut self, value: &Value, probes: usize) -> Result<()> {
        let name = match value {
            Value::Native(native) if native.properties.is_none() => Some(native.name.as_str()),
            Value::Json => Some("JSON"),
            Value::Math => Some("Math"),
            _ => None,
        };
        if let Some(name) = name {
            self.work(search(self.native_properties.len(), name.len()).saturating_mul(probes))?;
        }
        Ok(())
    }

    fn reflect_prototype(&mut self, value: &Value, doc: &Document) -> Result<Option<Value>> {
        self.work(2)?;
        self.reflect_registry_admission(value, 1)?;
        // The ordinary helper may select a fixed fallback intrinsic prototype.
        // DOM's interface/override route already admits its own table probes.
        if dom_own_properties::host(value).is_none() && self.property_object(value).is_none() {
            self.work(search(self.prototypes.len(), 11))?;
            // property_object is called again inside prototype_of_in.
            self.reflect_registry_admission(value, 1)?;
        } else if matches!(value, Value::Native(_) | Value::Json | Value::Math) {
            self.reflect_registry_admission(value, 1)?;
        }
        self.prototype_of_in(value, doc)
    }

    // The second result means a canonical numeric key has completed its exotic
    // lookup, including absence. It must never fall through to a prototype.
    fn reflect_own(
        &mut self,
        target: &Value,
        key: &PropertyKey,
        read_value: bool,
    ) -> Result<(Option<Property>, bool)> {
        self.work(8)?;
        if let PropertyKey::String(text) = key
            && let typed_array::Exotic::Handled(property) =
                self.typed_array_own_property(target, text, read_value)?
        {
            return Ok((property, true));
        }
        self.reflect_own_ordinary(target, key, read_value)
    }

    fn reflect_own_for_set(
        &mut self,
        target: &Value,
        key: &mut typed_array::SetKey<'_>,
        read_value: bool,
    ) -> Result<(Option<Property>, bool)> {
        self.work(8)?;
        if let typed_array::Exotic::Handled(property) =
            self.typed_array_own_property_for_set(target, key, read_value)?
        {
            return Ok((property, true));
        }
        self.reflect_own_ordinary(target, key.property(), read_value)
    }

    fn reflect_own_ordinary(
        &mut self,
        target: &Value,
        key: &PropertyKey,
        read_value: bool,
    ) -> Result<(Option<Property>, bool)> {
        if let PropertyKey::String(text) = key
            && target == &Value::Window
        {
            return self.window_reflected_property(text).map(|p| (p, false));
        }
        if dom_own_properties::host(target).is_some() {
            return self.read_own_property_key(target, key).map(|p| (p, false));
        }
        self.reflect_registry_admission(target, 2)?;
        let mut object = self.property_object(target);
        if object.is_none() && matches!(key, PropertyKey::Symbol(_)) {
            self.work(1 + search(self.host_symbol_objects.len(), 0))?;
            object = self.symbol_property_object(target);
        }
        let Some(id) = object else {
            return if matches!(key, PropertyKey::Symbol(_)) {
                Ok((None, false))
            } else {
                Err(ScriptError::unsupported(
                    "host own-property reflection is not implemented",
                ))
            };
        };
        let units = key.as_string().map_or(0, JsString::len);
        self.work(4 + search(self.objects[id].values.len(), units))?;
        if let Some(mut property) = self.objects[id].values.get(key).cloned() {
            if let PropertyValue::Data { value, .. } = &mut property.value {
                if read_value {
                    if let Some(text) = key.as_string() {
                        self.work(
                            search(self.objects[id].parameter_map.len(), units).saturating_mul(2),
                        )?;
                        if let Some((env, name)) = self.objects[id].parameter_map.get(text) {
                            let work = search(self.environments[*env].bindings.len(), name.len());
                            self.work(work)?;
                            // The admission above cannot run author code. Reborrow
                            // the stable mapping without copying its owned name.
                            let (env, name) = &self.objects[id].parameter_map[text];
                            *value = self.environments[*env].bindings[name].value.clone();
                        }
                    }
                } else {
                    *value = Value::Undefined;
                }
            }
            return Ok((Some(property), false));
        }
        let Some(text) = key.as_string() else {
            return Ok((None, false));
        };
        self.work(32 + text.len())?;
        let length = text.units() == [108, 101, 110, 103, 116, 104];
        let index = json_array_index(text).map(|i| i as usize);
        if let Value::Array(array) = target {
            if length {
                return Ok((
                    Some(Property::data(
                        Value::Number(self.array_lengths[*array].value as f64),
                        self.array_lengths[*array].writable,
                        false,
                        false,
                    )),
                    false,
                ));
            }
            if let Some(index) = index {
                self.work(search(self.array_holes[*array].len(), 0))?;
                if !self.array_holes[*array].contains(&index)
                    && let Some(value) = self.arrays[*array].get(index)
                {
                    return Ok((
                        Some(Property::data(
                            if read_value {
                                value.clone()
                            } else {
                                Value::Undefined
                            },
                            true,
                            true,
                            true,
                        )),
                        false,
                    ));
                }
            }
        }
        if let Some(Value::String(value)) = &self.objects[id].boxed {
            if length {
                return Ok((
                    Some(Property::data(
                        Value::Number(value.len() as f64),
                        false,
                        false,
                        false,
                    )),
                    false,
                ));
            }
            if let Some(unit) = index.and_then(|i| value.units().get(i)).copied() {
                let value = if read_value {
                    self.work(2)?;
                    self.charge(64)?;
                    Value::String(JsString::from(&[unit][..]))
                } else {
                    Value::Undefined
                };
                return Ok((Some(Property::data(value, false, true, false)), false));
            }
        }
        Ok((None, false))
    }

    fn reflect_find(
        &mut self,
        target: &Value,
        key: &PropertyKey,
        read_value: bool,
        doc: &Document,
    ) -> Result<Option<Property>> {
        let mut cursor = target.clone();
        let mut unresolved_dom_string = false;
        for _ in 0..MAX_DEPTH {
            self.work(2)?;
            let (property, terminal) = self.reflect_own(&cursor, key, read_value)?;
            if terminal || property.is_some() {
                return Ok(property);
            }
            unresolved_dom_string |= dom_own_properties::host(&cursor).is_some()
                && matches!(key, PropertyKey::String(_));
            let Some(next) = self.reflect_prototype(&cursor, doc)? else {
                if unresolved_dom_string {
                    return Err(ScriptError::unsupported(
                        "unrepresented DOM fallback reflection is not implemented",
                    ));
                }
                return Ok(None);
            };
            cursor = next;
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }

    fn reflect_set(
        &mut self,
        target: &Value,
        key: &PropertyKey,
        value: Value,
        receiver: Value,
        doc: &mut Document,
    ) -> Result<bool> {
        let mut classified = typed_array::SetKey::from_property(key);
        let mut cursor = target.clone();
        let mut unresolved_dom_string = false;
        for depth in 0..MAX_DEPTH {
            self.work(4)?;
            if let Some(classified) = classified.as_mut()
                && let typed_array::Exotic::Handled(result) = self.typed_array_set_for_set(
                    &cursor,
                    classified,
                    &receiver,
                    value.clone(),
                    doc,
                )?
            {
                return Ok(result);
            }
            let (property, terminal) = if let Some(classified) = classified.as_mut() {
                self.reflect_own_for_set(&cursor, classified, false)?
            } else {
                self.reflect_own(&cursor, key, false)?
            };
            if terminal && property.is_none() {
                // No callback lies between the two typed probes; normally the
                // preceding [[Set]] already completed this invalid-index case.
                return Ok(true);
            }
            if let Some(property) = property {
                match property.value {
                    PropertyValue::Accessor {
                        set: Value::Undefined,
                        ..
                    }
                    | PropertyValue::Data {
                        writable: false, ..
                    } => return Ok(false),
                    PropertyValue::Accessor { set, .. } => {
                        self.work(4)?;
                        self.charge(32 + std::mem::size_of::<Value>())?;
                        let mut args = Vec::new();
                        args.try_reserve_exact(1).map_err(|_| {
                            ScriptError::resource("Reflect setter arguments allocation failed")
                        })?;
                        args.push(value);
                        self.call(set, args, receiver, doc)?;
                        return Ok(true);
                    }
                    PropertyValue::Data { .. } => {}
                }
            } else {
                unresolved_dom_string |= dom_own_properties::host(&cursor).is_some()
                    && matches!(key, PropertyKey::String(_));
                if let Some(next) = self.reflect_prototype(&cursor, doc)? {
                    if depth + 1 == MAX_DEPTH {
                        return Err(ScriptError::resource("prototype chain limit exceeded"));
                    }
                    cursor = next;
                    continue;
                }
                if unresolved_dom_string {
                    return Err(ScriptError::unsupported(
                        "unrepresented DOM fallback reflection is not implemented",
                    ));
                }
            }
            if !js_object(&receiver) {
                return Ok(false);
            }
            let (own, _) = if let Some(classified) = classified.as_mut() {
                self.reflect_own_for_set(&receiver, classified, false)?
            } else {
                self.reflect_own(&receiver, key, false)?
            };
            let descriptor = match own {
                Some(Property {
                    value: PropertyValue::Accessor { .. },
                    ..
                })
                | Some(Property {
                    value:
                        PropertyValue::Data {
                            writable: false, ..
                        },
                    ..
                }) => return Ok(false),
                Some(_) => PropertyDescriptor {
                    value: Some(value),
                    ..PropertyDescriptor::default()
                },
                None => PropertyDescriptor::data_property(value, true, true, true),
            };
            // The shared descriptor helper retains its Array length callbacks,
            // TypedArray fresh checks, mutation admissions and Boolean result.
            return if let Some(classified) = classified.as_mut() {
                self.define_property_key_for_set(&receiver, classified, descriptor, doc)
            } else {
                self.define_property_key(&receiver, key, descriptor, doc)
            };
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }

    // The intrinsic registry survives replacement of the global Object binding.
    // This lookup creates no owned storage and precedes any prototype mutation.
    pub(super) fn object_prototype_is_immutable(&mut self, id: usize) -> Result<bool> {
        self.work(2 + search(self.prototypes.len(), 6))?;
        Ok(self
            .prototypes
            .get("Object")
            .is_some_and(|prototype| *prototype == id))
    }

    fn reflect_set_prototype(
        &mut self,
        target: &Value,
        next: Value,
        doc: &Document,
    ) -> Result<bool> {
        self.reflect_registry_admission(target, 1)?;
        let id = self.property_object(target).ok_or_else(|| {
            ScriptError::unsupported("host object prototype mutation is unsupported")
        })?;
        self.work(6)?;
        let next = if next == Value::Null {
            None
        } else {
            Some(next)
        };
        if self.objects[id].prototype == next {
            return Ok(true);
        }
        if self.object_prototype_is_immutable(id)? || self.objects[id].non_extensible {
            return Ok(false);
        }
        let mut cursor = next.clone();
        for depth in 0..=MAX_DEPTH {
            self.work(2)?;
            let Some(value) = cursor else {
                // No fallible operation or budget debit follows publication.
                self.objects[id].prototype = next;
                return Ok(true);
            };
            if &value == target {
                return Ok(false);
            }
            if depth == MAX_DEPTH {
                return Err(ScriptError::resource("prototype chain limit exceeded"));
            }
            cursor = self.reflect_prototype(&value, doc)?;
        }
        unreachable!()
    }

    fn reflect_descriptor(&mut self, property: Property) -> Result<Value> {
        // Four fixed fields: admit the fresh leaf, order block, bounded map
        // searches/moves and possible object-arena relocation before creating it.
        // insert_property does both contains_key and insert: twelve bounded
        // comparisons, not six, for the four-field construction.
        self.work(276)?;
        self.charge(
            16 * (std::mem::size_of::<PropertyKey>() + std::mem::size_of::<Property>())
                + 32 * std::mem::size_of::<usize>()
                + 64
                + 4 * std::mem::size_of::<PropertyKey>(),
        )?;
        if self.objects.len() == self.objects.capacity() {
            let count = self.objects.len();
            let bytes = count
                .checked_add(1)
                .and_then(|n| n.checked_mul(std::mem::size_of::<ScriptObject>()))
                .ok_or_else(|| ScriptError::resource("Reflect descriptor arena overflow"))?;
            self.work(1 + count.saturating_mul(2))?;
            self.charge(bytes)?;
            self.objects
                .try_reserve_exact(1)
                .map_err(|_| ScriptError::resource("Reflect descriptor arena allocation failed"))?;
        }
        self.work(search(self.prototypes.len(), 6))?;
        let fields = match property.value {
            PropertyValue::Data { value, writable } => [
                (self.dom_proto_text("value")?, value),
                (self.dom_proto_text("writable")?, Value::Bool(writable)),
                (
                    self.dom_proto_text("enumerable")?,
                    Value::Bool(property.enumerable),
                ),
                (
                    self.dom_proto_text("configurable")?,
                    Value::Bool(property.configurable),
                ),
            ],
            PropertyValue::Accessor { get, set } => [
                (self.dom_proto_text("get")?, get),
                (self.dom_proto_text("set")?, set),
                (
                    self.dom_proto_text("enumerable")?,
                    Value::Bool(property.enumerable),
                ),
                (
                    self.dom_proto_text("configurable")?,
                    Value::Bool(property.configurable),
                ),
            ],
        };
        self.object_ordered(fields)
    }
}
