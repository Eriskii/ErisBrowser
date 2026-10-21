//! Immutable home identity for ordinary object-literal methods and accessors.
use super::*;

fn invalid() -> ScriptError {
    ScriptError::resource("invalid object method home state")
}

impl Runtime {
    pub(super) fn object_attach_home(&mut self, method: &Value, home: &Value) -> Result<()> {
        // Admit the bounded checks and one scalar publication before any mutation.
        // function_value already paid for the home slot; this retains no new storage.
        self.work(8)?;
        let (Value::Function(id), Value::Object(home)) = (method, home) else {
            return Err(invalid());
        };
        if *home >= self.objects.len() {
            return Err(invalid());
        }
        let function = self.functions.get_mut(*id).ok_or_else(invalid)?;
        if function.bound.is_some()
            || function.home_object.is_some()
            || function.code.arrow
            || function.code.constructable
            || function.code.class_kind.is_some()
        {
            return Err(invalid());
        }
        // Object methods preserve their source's strict or sloppy this semantics.
        function.home_object = Some(*home);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
