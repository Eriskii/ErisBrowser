//! Persistent class home identity and ordered SuperProperty capture.
use super::*;

fn invalid() -> ScriptError {
    ScriptError::resource("invalid super property runtime state")
}

impl Runtime {
    pub(super) fn class_attach_home(&mut self, method: &Value, home: usize) -> Result<()> {
        // Kind/range/metadata checks and the one immutable scalar publication.
        self.work(8)?;
        let Value::Function(id) = method else {
            return Err(invalid());
        };
        if home >= self.objects.len() {
            return Err(invalid());
        }
        let function = self.functions.get_mut(*id).ok_or_else(invalid)?;
        if function.bound.is_some()
            || function.home_object.is_some()
            || function.code.arrow
            || function.code.constructable
            || !function.code.strict
            || function.code.class_kind.is_some()
        {
            return Err(invalid());
        }
        function.home_object = Some(home);
        Ok(())
    }

    pub(super) fn class_super_property_owner(
        &mut self,
        environment: usize,
    ) -> Result<(usize, Value)> {
        // Validate ownership and prepay the receiver retain, handoff and cleanup.
        // Each lexical environment hop retains its existing separate tick.
        self.work(8)?;
        let owner = self.this_environment(environment)?;
        let receiver = self.class_this_value(owner)?;
        let row = self.environments.get(owner).ok_or_else(invalid)?;
        let function = self
            .functions
            .get(row.active_function.ok_or_else(invalid)?)
            .ok_or_else(invalid)?;
        let home = function.home_object.ok_or_else(invalid)?;
        if function.bound.is_some() || function.code.arrow || home >= self.objects.len() {
            return Err(invalid());
        }
        Ok((owner, receiver))
    }

    pub(super) fn class_super_property_reference(
        &mut self,
        owner: usize,
        receiver: Value,
        name: Value,
        strict: bool,
    ) -> Result<Reference> {
        // Initial capture/child scheduling funded incoming owners on refusal.
        // Revalidate IDs after key evaluation, retain the live base, then assemble.
        self.work(8)?;
        let row = self.environments.get(owner).ok_or_else(invalid)?;
        let function = self
            .functions
            .get(row.active_function.ok_or_else(invalid)?)
            .ok_or_else(invalid)?;
        if function.bound.is_some() || function.code.arrow {
            return Err(invalid());
        }
        let home = self
            .objects
            .get(function.home_object.ok_or_else(invalid)?)
            .ok_or_else(invalid)?;
        let base = home.prototype.as_ref().unwrap_or(&Value::Null);
        // This bounded kind/range stage also rejects malformed runtime arena IDs
        // before the shared property helpers can index their own storage.
        let valid = match base {
            Value::Object(id) => *id < self.objects.len(),
            Value::Function(id) => self
                .functions
                .get(*id)
                .is_some_and(|function| function.properties < self.objects.len()),
            Value::Array(id) => {
                *id < self.arrays.len()
                    && self
                        .array_properties
                        .get(*id)
                        .is_some_and(|bag| *bag < self.objects.len())
            }
            Value::Native(native) => native
                .properties
                .is_none_or(|bag| bag.get() < self.objects.len()),
            _ => base == &Value::Null || js_object(base),
        };
        if !valid {
            return Err(invalid());
        }
        Ok(Reference::SuperProperty {
            base: base.clone(),
            name,
            receiver,
            strict,
        })
    }
}
#[cfg(test)]
mod tests;
