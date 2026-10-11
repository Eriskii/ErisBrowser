//! Best-effort thrown-value diagnostics from prepaid, stored data descriptors.
//!
//! This is not ordinary Get: diagnostic accessors never run, and a blocked or
//! unsupported lookup terminates without falling through to another descriptor.
use super::*;

#[derive(Debug)]
pub(super) struct ThrowMetadata {
    pub(super) name: Option<String>,
    pub(super) intrinsic: Option<&'static str>,
}

fn stored_metadata(value: &Value) -> bool {
    matches!(
        value,
        Value::Object(_)
            | Value::Function(_)
            | Value::Array(_)
            | Value::Native(_)
            | Value::Math
            | Value::Json
    )
}

// Existing pinned B=6 comparison envelope. Units are logical comparison blocks,
// not measured instructions. Fixed property keys and registry names are borrowed.
fn search_work(count: usize, units: usize) -> Result<usize> {
    if count == 0 {
        return Ok(0);
    }
    let overflow = || ScriptError::resource("throw metadata work overflow");
    let mut capacity = count.checked_add(1).ok_or_else(overflow)? / 2;
    let mut height = 1usize;
    while capacity >= 6 {
        capacity /= 6;
        height = height.checked_add(1).ok_or_else(overflow)?;
    }
    let comparisons = count.min(height.checked_mul(11).ok_or_else(overflow)?);
    comparisons
        .checked_mul((units / 8).checked_add(1).ok_or_else(overflow)?)
        .ok_or_else(overflow)
}

impl Runtime {
    pub(super) fn throw_metadata(&mut self, value: &Value) -> Result<ThrowMetadata> {
        // Covers classification and bounded retained-handle/final-result cleanup.
        self.work(8)?;
        if !stored_metadata(value) {
            return Ok(ThrowMetadata {
                name: None,
                intrinsic: None,
            });
        }
        let brand = match value {
            Value::Object(id) => self
                .objects
                .get(*id)
                .and_then(|object| object.intrinsic_error),
            _ => None,
        };
        let key = PropertyKey::String(self.dom_proto_text("constructor")?);
        let Some(constructor) = self.throw_metadata_data(value, &key, 11)? else {
            return Ok(ThrowMetadata {
                name: None,
                intrinsic: None,
            });
        };
        // One constructor snapshot serves both classifications. Name metadata is
        // optional and does not erase a known private brand/canonical constructor.
        let name = if json_callable(&constructor) {
            let key = PropertyKey::String(self.dom_proto_text("name")?);
            if let Some(Value::String(name)) = self.throw_metadata_data(&constructor, &key, 4)? {
                self.work(1 + name.len() / 16)?;
                let bytes = name
                    .len()
                    .checked_mul(3)
                    .and_then(|n| n.checked_add(24))
                    .ok_or_else(|| ScriptError::resource("throw metadata string overflow"))?;
                self.charge(bytes)?;
                Some(name.to_utf8_lossy())
            } else {
                None
            }
        } else {
            None
        };
        let intrinsic = if let Some(brand) = brand {
            self.work(
                brand
                    .len()
                    .checked_add(8)
                    .ok_or_else(|| ScriptError::resource("throw metadata work overflow"))?,
            )?;
            // Native identity is its internal name and stored receiver. Do not
            // allocate a comparison Native or inspect mutable metadata/global names.
            matches!(&constructor, Value::Native(native)
                if native.name == brand && matches!(native.receiver, Value::Window))
            .then_some(brand)
        } else {
            None
        };
        Ok(ThrowMetadata { name, intrinsic })
    }

    fn throw_metadata_bag(&mut self, value: &Value) -> Result<Option<usize>> {
        // Caller has paid the fixed edge envelope before direct arena access.
        let registry_key = match value {
            Value::Object(id) => return Ok(Some(*id)),
            Value::Function(id) => {
                return Ok(self.functions.get(*id).map(|function| function.properties));
            }
            Value::Array(id) => return Ok(self.array_properties.get(*id).copied()),
            Value::Native(native) => {
                if let Some(id) = native.properties {
                    return Ok(Some(id.get()));
                }
                native.name.as_str()
            }
            Value::Math => "Math",
            Value::Json => "JSON",
            _ => return Ok(None),
        };
        self.work(search_work(
            self.native_properties.len(),
            registry_key.len(),
        )?)?;
        // A missing registry bag is outside this best-effort metadata domain.
        // In particular, do not silently traverse Native's Function prototype.
        Ok(self.native_properties.get(registry_key).copied())
    }

    fn throw_metadata_data(
        &mut self,
        value: &Value,
        key: &PropertyKey,
        units: usize,
    ) -> Result<Option<Value>> {
        let mut cursor = value.clone();
        for _ in 0..MAX_DEPTH {
            // Covers edge setup/bounds, handle retention and cleanup. No author
            // callback or recursive payload copy occurs during this walk.
            self.work(8)?;
            let Some(id) = self.throw_metadata_bag(&cursor)? else {
                return Ok(None);
            };
            let Some(count) = self.objects.get(id).map(|object| object.values.len()) else {
                return Ok(None);
            };
            self.work(search_work(count, units)?)?;
            let object = &self.objects[id];
            if let Some(property) = object.values.get(key) {
                return Ok(match &property.value {
                    PropertyValue::Data { value, .. } => Some(value.clone()),
                    PropertyValue::Accessor { .. } => None,
                });
            }
            let Some(prototype) = &object.prototype else {
                return Ok(None);
            };
            cursor = prototype.clone();
        }
        Err(ScriptError::resource("prototype chain limit exceeded"))
    }
}
