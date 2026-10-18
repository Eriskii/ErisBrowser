//! Super property lookup keeps the captured base separate from actual `this`.
//! Key conversion and retained-reference ownership belong to GetValue/PutValue.
use super::*;

#[cfg(test)]
mod tests;

impl Runtime {
    pub(super) fn get_super_property_key(
        &mut self,
        base: &Value,
        key: &PropertyKey,
        receiver: &Value,
        doc: &mut Document,
    ) -> Result<Value> {
        // Guard, cursor owner, lookup handoff, descriptor/result ownership,
        // result routing and cleanup. Core lookup tariffs remain unchanged.
        self.work(6)?;
        if !js_object(base) {
            return Err(ScriptError::type_error(
                "super property base must be an object",
            ));
        }
        match self.reflect_find(base, key, true, doc)? {
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
            }) => {
                self.work(1)?;
                self.call(get, Vec::new(), receiver.clone(), doc)
            }
        }
    }

    pub(super) fn set_super_property_key(
        &mut self,
        base: &Value,
        key: &PropertyKey,
        receiver: &Value,
        value: Value,
        strict: bool,
        doc: &mut Document,
    ) -> Result<()> {
        // The caller funds incoming RHS release if this first debit refuses.
        // No receiver lookup or author callback precedes wrapper admission.
        self.work(6)?;
        if !js_object(base) {
            return Err(ScriptError::type_error(
                "super property base must be an object",
            ));
        }
        self.work(1)?;
        if self.reflect_set(base, key, value, receiver.clone(), doc)? {
            Ok(())
        } else {
            Self::failed_write(strict)
        }
    }
}
