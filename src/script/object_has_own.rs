//! Object.hasOwn over the runtime's represented own-property descriptors.
use super::*;

#[cfg(test)]
mod tests;

pub(super) const METADATA_OBJECTS: usize = 1;
const NAME: &str = "Object.hasOwn";

fn tree(count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let mut capacity = count.saturating_add(1) / 2;
    let mut height = 1usize;
    while capacity >= 6 {
        capacity /= 6;
        height += 1;
    }
    (count.min(11 * height), height)
}

impl Runtime {
    pub(super) fn install_object_has_own_intrinsic(&mut self) -> Result<()> {
        self.work(8 + tree(self.native_properties.len()).0 * 7)?;
        let owner = *self
            .native_properties
            .get("Object")
            .ok_or_else(|| ScriptError::resource("Object.hasOwn constructor metadata missing"))?;
        let properties =
            self.dom_proto_object(Some(Value::Function(self.function_prototype)), 2)?;
        let display = self.dom_proto_text("hasOwn")?;
        self.dom_proto_named(
            properties,
            "length",
            Property::data(Value::Number(2.0), false, false, true),
        )?;
        self.dom_proto_named(
            properties,
            "name",
            Property::data(Value::String(display), false, false, true),
        )?;
        self.work(8 + NAME.len())?;
        self.charge(std::mem::size_of::<Native>() + 32 + NAME.len())?;
        let mut name = String::new();
        name.try_reserve_exact(NAME.len())
            .map_err(|_| ScriptError::resource("Object.hasOwn native name allocation failed"))?;
        name.push_str(NAME);
        let properties = std::num::NonZeroUsize::new(properties)
            .ok_or_else(|| ScriptError::resource("Object.hasOwn metadata ID is zero"))?;
        let function = Value::Native(Rc::new(Native {
            name,
            properties: Some(properties),
            receiver: Value::Undefined,
        }));
        // Supplement only this new insertion: the shared helper still pays its
        // original movement row, typed tree blocks and actual order growth.
        let count = self.objects[owner].values.len();
        let levels = tree(count).1 + 1;
        let old_moves = if count < 11 { count + 8 } else { 28 * levels };
        self.work((256 * levels + 32).saturating_sub(old_moves))?;
        self.dom_proto_named(owner, "hasOwn", Property::data(function, true, false, true))
    }

    pub(super) fn object_has_own(&mut self, args: &[Value], doc: &mut Document) -> Result<Value> {
        self.work(8)?;
        let target = args.first().cloned().unwrap_or(Value::Undefined);
        if !js_object(&target) && !matches!(target, Value::Null | Value::Undefined) {
            // coerce_object performs the primitive-prototype and Object lookups.
            self.work(8 + 2 * tree(self.prototypes.len()).0 * 8)?;
            if self.objects.len() == self.objects.capacity() {
                let count = self.objects.len();
                let bytes = count
                    .checked_add(1)
                    .and_then(|capacity| capacity.checked_mul(std::mem::size_of::<ScriptObject>()))
                    .ok_or_else(|| {
                        ScriptError::resource("Object.hasOwn boxing storage overflow")
                    })?;
                self.work(1usize.saturating_add(count.saturating_mul(2)))?;
                self.charge(bytes)?;
                self.objects.try_reserve_exact(1).map_err(|_| {
                    ScriptError::resource("Object.hasOwn boxing arena allocation failed")
                })?;
            }
        }
        let object = self.coerce_object(target)?;
        let key = self.property_key(args.get(1).cloned().unwrap_or(Value::Undefined), doc)?;
        self.work(4)?;
        let present = if object == Value::Window && key.as_string().is_some() {
            self.window_reflected_property(key.as_string().unwrap())?
                .is_some()
        } else if dom_own_properties::host(&object).is_some() {
            self.read_own_property_key(&object, &key)?.is_some()
        } else {
            self.object_has_own_present(&object, &key)?
        };
        self.work(2)?;
        Ok(Value::Bool(present))
    }

    fn object_has_own_present(&mut self, receiver: &Value, key: &PropertyKey) -> Result<bool> {
        if let PropertyKey::String(name) = key
            && let typed_array::Exotic::Handled(property) =
                self.typed_array_own_property(receiver, name, false)?
        {
            return Ok(property.is_some());
        }
        let registry_name = match receiver {
            Value::Native(native) if native.properties.is_none() => Some(native.name.as_str()),
            Value::Math => Some("Math"),
            Value::Json => Some("JSON"),
            _ => None,
        };
        if let Some(name) = registry_name {
            self.work(
                2usize
                    .saturating_mul(tree(self.native_properties.len()).0)
                    .saturating_mul(1 + name.len()),
            )?;
        }
        let mut object = self.property_object(receiver);
        if matches!(key, PropertyKey::Symbol(_)) && object.is_none() {
            self.work(1 + tree(self.host_symbol_objects.len()).0)?;
            object = self.symbol_property_object(receiver);
        }
        let Some(object) = object else {
            return if matches!(key, PropertyKey::Symbol(_)) {
                Ok(false)
            } else {
                Err(ScriptError::unsupported(
                    "host own-property reflection is not implemented",
                ))
            };
        };
        self.work(
            1 + tree(self.objects[object].values.len())
                .0
                .saturating_mul(1 + key.as_string().map_or(0, JsString::len)),
        )?;
        // Descriptor presence alone does not read getters, mapped parameter
        // values or materialize a discarded virtual string-index character.
        if self.objects[object].values.contains_key(key) {
            return Ok(true);
        }
        let Some(key) = key.as_string() else {
            return Ok(false);
        };
        self.work(32)?;
        let length = key.units() == [108, 101, 110, 103, 116, 104];
        let index = json_array_index(key).map(|index| index as usize);
        if let Value::Array(id) = receiver {
            if length {
                return Ok(true);
            }
            if let Some(index) = index {
                self.work(tree(self.array_holes[*id].len()).0)?;
                if index < self.arrays[*id].len() && !self.array_holes[*id].contains(&index) {
                    return Ok(true);
                }
            }
        }
        Ok(matches!(
            &self.objects[object].boxed,
            Some(Value::String(text)) if length || index.is_some_and(|index| index < text.len())
        ))
    }
}
