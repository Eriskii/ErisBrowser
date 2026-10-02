//! Property operations that preserve string and symbol key identity.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum HostKey {
    Window,
    Document,
    Console,
    Node(NodeId),
    Style(NodeId),
    ClassList(NodeId),
}
impl HostKey {
    fn of(value: &Value) -> Option<Self> {
        Some(match value {
            Value::Window => Self::Window,
            Value::Document => Self::Document,
            Value::Console => Self::Console,
            Value::Node(id) => Self::Node(*id),
            Value::Style(id) => Self::Style(*id),
            Value::ClassList(id) => Self::ClassList(*id),
            _ => return None,
        })
    }
}

impl Runtime {
    pub(super) fn symbol_property_object(&self, receiver: &Value) -> Option<usize> {
        self.property_object(receiver).or_else(|| {
            HostKey::of(receiver).and_then(|key| self.host_symbol_objects.get(&key).copied())
        })
    }

    pub(super) fn ensure_symbol_property_object(
        &mut self,
        receiver: &Value,
    ) -> Result<Option<usize>> {
        if let Some(id) = self.symbol_property_object(receiver) {
            return Ok(Some(id));
        }
        if !js_object(receiver) {
            return Ok(None);
        }
        let host = HostKey::of(receiver);
        let native = if let Value::Native(native) = receiver {
            Some(&native.name)
        } else {
            None
        };
        if host.is_none() && native.is_none() {
            return Err(ScriptError::unsupported(
                "host symbol properties are not implemented",
            ));
        }
        self.charge(128 + native.map_or(0, String::len))?;
        let Value::Object(id) = self.object_ordered([])? else {
            unreachable!()
        };
        if let Some(key) = host {
            self.host_symbol_objects.insert(key, id);
        }
        if let Some(name) = native {
            self.objects[id].prototype = Some(Value::Function(self.function_prototype));
            self.native_properties.insert(name.clone(), id);
        }
        Ok(Some(id))
    }

    pub(super) fn own_property_key(&self, receiver: &Value, key: &PropertyKey) -> Option<Property> {
        match key {
            PropertyKey::String(key) => self.own_property(receiver, key),
            PropertyKey::Symbol(_) => self
                .symbol_property_object(receiver)
                .and_then(|id| self.objects[id].values.get(key).cloned()),
        }
    }

    pub(super) fn find_property_key(
        &mut self,
        receiver: &Value,
        key: &PropertyKey,
    ) -> Result<Option<Property>> {
        if let PropertyKey::String(key) = key {
            return self.find_property(receiver, key);
        }
        let mut cursor = Some(receiver.clone());
        for _ in 0..MAX_DEPTH {
            let Some(value) = cursor else { return Ok(None) };
            self.tick()?;
            if let Some(property) = self.read_own_property_key(&value, key)? {
                return Ok(Some(property));
            }
            cursor = self.prototype_of(&value);
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }

    pub(super) fn get_property_key(
        &mut self,
        receiver: Value,
        key: &PropertyKey,
        doc: &mut Document,
    ) -> Result<Value> {
        if let PropertyKey::String(key) = key {
            return self.get_key(receiver, key, doc);
        }
        if matches!(receiver, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "cannot read property of null or undefined",
            ));
        }
        let Some(property) = self.find_property_key(&receiver, key)? else {
            return Ok(Value::Undefined);
        };
        match property.value {
            PropertyValue::Data { value, .. } => Ok(value),
            PropertyValue::Accessor {
                get: Value::Undefined,
                ..
            } => Ok(Value::Undefined),
            PropertyValue::Accessor { get, .. } => self.call(get, Vec::new(), receiver, doc),
        }
    }

    pub(super) fn set_property_key(
        &mut self,
        receiver: Value,
        key: &PropertyKey,
        value: Value,
        strict: bool,
        doc: &mut Document,
    ) -> Result<()> {
        if let PropertyKey::String(key) = key {
            return self.set_key_strict(receiver, key, value, strict, doc);
        }
        if matches!(receiver, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "cannot write property of null or undefined",
            ));
        }
        if let Some(property) = self.find_property_key(&receiver, key)? {
            match property.value {
                PropertyValue::Accessor {
                    set: Value::Undefined,
                    ..
                }
                | PropertyValue::Data {
                    writable: false, ..
                } => return Self::failed_write(strict),
                PropertyValue::Accessor { set, .. } => {
                    if dom_own_properties::host(&receiver).is_some() {
                        self.dom_call_setter(set, value, receiver, doc)?;
                    } else {
                        self.call(set, vec![value], receiver, doc)?;
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
        if !js_object(&receiver) {
            return Self::failed_write(strict);
        }
        let desc = if self.read_own_property_key(&receiver, key)?.is_some() {
            PropertyDescriptor {
                value: Some(value),
                ..PropertyDescriptor::default()
            }
        } else {
            PropertyDescriptor::data_property(value, true, true, true)
        };
        if self.define_own_key(&receiver, key, desc)? {
            Ok(())
        } else {
            Self::failed_write(strict)
        }
    }

    pub(super) fn delete_property_key(
        &mut self,
        receiver: Value,
        key: &PropertyKey,
    ) -> Result<bool> {
        if dom_own_properties::host(&receiver).is_some() {
            return self.dom_delete_own(&receiver, key);
        }
        if let PropertyKey::String(key) = key {
            return self.delete_property(receiver, key);
        }
        self.tick()?;
        if matches!(receiver, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "cannot delete property of null or undefined",
            ));
        }
        let Some(property) = self.own_property_key(&receiver, key) else {
            return Ok(true);
        };
        if !property.configurable {
            return Ok(false);
        }
        if let Some(id) = self.symbol_property_object(&receiver) {
            self.work(1 + self.objects[id].order.len() / 8)?;
            self.objects[id].remove(key);
        }
        Ok(true)
    }

    pub(super) fn own_symbol_keys(&mut self, receiver: &Value) -> Result<Vec<PropertyKey>> {
        let mut result = Vec::new();
        let object = if dom_own_properties::host(receiver).is_some() {
            self.dom_own_object(receiver)?
        } else {
            self.symbol_property_object(receiver)
        };
        if let Some(id) = object {
            self.work(self.objects[id].order.len() + 1)?;
            let count = self.objects[id]
                .order
                .iter()
                .filter(|key| matches!(key, PropertyKey::Symbol(_)))
                .count();
            self.charge(count.saturating_mul(std::mem::size_of::<PropertyKey>()) + 32)?;
            result
                .try_reserve_exact(count)
                .map_err(|_| ScriptError::resource("symbol key list allocation failed"))?;
            result.extend(
                self.objects[id]
                    .order
                    .iter()
                    .filter(|key| matches!(key, PropertyKey::Symbol(_)))
                    .cloned(),
            );
        }
        Ok(result)
    }

    pub(super) fn own_property_keys(&mut self, receiver: &Value) -> Result<Vec<PropertyKey>> {
        let strings = self.own_keys(receiver)?;
        let symbols = self.own_symbol_keys(receiver)?;
        let length = strings.len().saturating_add(symbols.len());
        self.work(length + 1)?;
        self.charge(length.saturating_mul(std::mem::size_of::<PropertyKey>()) + 32)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("property key list allocation failed"))?;
        result.extend(strings.into_iter().map(PropertyKey::String));
        result.extend(symbols);
        Ok(result)
    }
}
