//! Integrity levels on supported ordinary objects and array/string exotics.
use super::*;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub(super) struct IntegrityOperation {
    frozen: bool,
    query: bool,
}

impl IntegrityOperation {
    pub(super) fn from_name(name: &str) -> Option<Self> {
        let (frozen, query) = match name {
            "Object.seal" => (false, false),
            "Object.freeze" => (true, false),
            "Object.isSealed" => (false, true),
            "Object.isFrozen" => (true, true),
            _ => return None,
        };
        Some(Self { frozen, query })
    }
}

// Numeric keys sort before other strings, then symbols. The insertion ordinal
// makes the total order explicit, so sorting needs no stable-sort allocation.
type IntegrityKey = (u8, u32, usize, PropertyKey);

fn push_key(keys: &mut Vec<IntegrityKey>, key: PropertyKey) {
    let (kind, index) = match &key {
        PropertyKey::String(string) => json_array_index(string).map_or((1, 0), |index| (0, index)),
        PropertyKey::Symbol(_) => (2, 0),
    };
    keys.push((kind, index, keys.len(), key));
}

impl Runtime {
    fn integrity_keys(&mut self, receiver: &Value, object: usize) -> Result<Vec<IntegrityKey>> {
        let dense = match receiver {
            Value::Array(id) => self.arrays[*id].len(),
            _ => 0,
        };
        let text = match &self.objects[object].boxed {
            Some(Value::String(text)) => Some(text.len()),
            _ => None,
        };
        let virtual_length = matches!(receiver, Value::Array(_)) || text.is_some();
        let virtual_indices = dense.saturating_add(text.unwrap_or(0));
        let bound = virtual_indices
            .saturating_add(usize::from(virtual_length))
            .saturating_add(self.objects[object].order.len());
        let logarithm = 1 + bound.checked_ilog2().unwrap_or(0) as usize;
        // Cover scanning dense-cache holes, short numeric-key lookups and the
        // in-place sort before materializing anything. Logical gaps are absent.
        self.work(1 + bound.saturating_mul(4 * logarithm))?;
        self.charge(
            64usize
                .saturating_add(bound.saturating_mul(std::mem::size_of::<IntegrityKey>()))
                .saturating_add(virtual_indices.saturating_mul(128)),
        )?;
        let mut keys = Vec::new();
        keys.try_reserve_exact(bound)
            .map_err(|_| ScriptError::resource("integrity key allocation failed"))?;
        for index in 0..virtual_indices {
            if let Value::Array(id) = receiver
                && self.array_holes[*id].contains(&index)
            {
                continue;
            }
            let key = PropertyKey::String(JsString::from(index.to_string()));
            // Stored overrides will be appended once from the creation order.
            if !self.objects[object].values.contains_key(&key) {
                push_key(&mut keys, key);
            }
        }
        if virtual_length {
            push_key(&mut keys, "length".into());
        }
        for key in &self.objects[object].order {
            if virtual_length
                && key
                    .as_string()
                    .is_some_and(|s| s.units() == [108, 101, 110, 103, 116, 104])
            {
                continue;
            }
            push_key(&mut keys, key.clone());
        }
        keys.sort_unstable_by_key(|(kind, index, ordinal, _)| (*kind, *index, *ordinal));
        Ok(keys)
    }

    // Read only flags: no accessor calls, mapped-value fetches or allocation
    // of the one-unit string value of a virtual boxed-string index.
    fn integrity_flags(
        &mut self,
        receiver: &Value,
        object: usize,
        key: &PropertyKey,
    ) -> Result<Option<(bool, Option<bool>)>> {
        let logarithm = 1 + self.objects[object]
            .values
            .len()
            .checked_ilog2()
            .unwrap_or(0) as usize;
        self.work(1 + (1 + key.byte_len()).saturating_mul(logarithm))?;
        if let Some(property) = self.objects[object].values.get(key) {
            return Ok(Some((
                property.configurable,
                match property.value {
                    PropertyValue::Data { writable, .. } => Some(writable),
                    PropertyValue::Accessor { .. } => None,
                },
            )));
        }
        let Some(key) = key.as_string() else {
            return Ok(None);
        };
        let length = key.units() == [108, 101, 110, 103, 116, 104];
        if let Value::Array(id) = receiver {
            if length {
                return Ok(Some((false, Some(self.array_lengths[*id].writable))));
            }
            if let Some(index) = json_array_index(key).map(|index| index as usize)
                && index < self.arrays[*id].len()
                && !self.array_holes[*id].contains(&index)
            {
                return Ok(Some((true, Some(true))));
            }
        }
        if let Some(Value::String(text)) = &self.objects[object].boxed
            && (length || json_array_index(key).is_some_and(|index| (index as usize) < text.len()))
        {
            return Ok(Some((false, Some(false))));
        }
        Ok(None)
    }

    pub(super) fn object_integrity(
        &mut self,
        receiver: Value,
        operation: IntegrityOperation,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        if !js_object(&receiver) {
            return Ok(if operation.query {
                Value::Bool(true)
            } else {
                receiver
            });
        }
        let object = self
            .property_object(&receiver)
            .ok_or_else(|| ScriptError::unsupported("host object integrity is not implemented"))?;
        if operation.query {
            if !self.objects[object].non_extensible {
                return Ok(Value::Bool(false));
            }
        } else {
            // This effect precedes the key snapshot and persists if a later
            // resource/definition failure interrupts progress.
            self.objects[object].non_extensible = true;
        }
        let keys = self.integrity_keys(&receiver, object)?;
        for (_, _, _, key) in keys {
            let Some((configurable, writable)) = self.integrity_flags(&receiver, object, &key)?
            else {
                continue;
            };
            let needs_change = configurable || (operation.frozen && writable == Some(true));
            if operation.query {
                if needs_change {
                    return Ok(Value::Bool(false));
                }
                continue;
            }
            // These targets have no Proxy/host definition trap. Compatible
            // unchanged flags need no table entry, notably virtual strings.
            if !needs_change {
                continue;
            }
            self.integrity_define(
                &receiver,
                object,
                &key,
                operation.frozen && writable.is_some(),
                doc,
            )?;
        }
        Ok(if operation.query {
            Value::Bool(true)
        } else {
            receiver
        })
    }

    fn integrity_define(
        &mut self,
        receiver: &Value,
        object: usize,
        key: &PropertyKey,
        readonly: bool,
        doc: &mut Document,
    ) -> Result<()> {
        if let Some((env, name)) = key
            .as_string()
            .and_then(|key| self.objects[object].parameter_map.get(key))
        {
            // Definition snapshots mapped arguments and may update their
            // binding. Name comparisons are not bounded by the index key.
            let log = 1 + self.environments[*env]
                .bindings
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize;
            self.work((1 + name.len()).saturating_mul(4 * log))?;
        }
        // Array length definition creates constant key scratch. Ordinary
        // new descriptor records retain their existing separate charge.
        self.charge(128)?;
        let descriptor = PropertyDescriptor {
            configurable: Some(false),
            writable: readonly.then_some(false),
            ..PropertyDescriptor::default()
        };
        if !self.define_property_key(receiver, key, descriptor, doc)? {
            return Err(ScriptError::type_error("cannot set object integrity level"));
        }
        Ok(())
    }
}
