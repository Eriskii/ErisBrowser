//! Number TypedArray copying reverse with saved intrinsic construction.
use super::*;

impl Runtime {
    pub(super) fn typed_array_to_reversed(
        &mut self,
        record: Record,
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8)?;
        // Validate even an empty source before creating a private result shell.
        let length = self.typed_array_length(record)?;
        // Admit intrinsic selection, handle lifetime, and the one stack argument.
        // The existing backing limit keeps this integer exactly representable.
        self.work(8)?;
        let constructor = self.typed_arrays.constructors[record.kind.index()]
            .clone()
            .ok_or_else(|| ScriptError::resource("TypedArray intrinsic missing"))?;
        // Its own .prototype is immutable data. This primitive-length branch
        // cannot call authors and preserves the existing zeroing/reserve fees.
        let result = self.typed_array_constructor(
            record.kind,
            &[Value::Number(length as f64)],
            constructor,
            doc,
        )?;
        // Keep only copied records across allocation and table growth. Retain
        // the existing paid authentication and bounds validation for the result.
        let target = self
            .typed_array_record(&result)?
            .ok_or_else(|| ScriptError::resource("TypedArray result missing"))?;
        let target_length = self.typed_array_length(target)?;
        self.work(4)?;
        if target.kind != record.kind || target_length != length {
            return Err(ScriptError::resource("TypedArray result shape mismatch"));
        }
        let mut index = 0;
        while index < length {
            // index < length proves both subtractions. Every position, including
            // an odd midpoint, goes through numeric decode and encode.
            self.work(8)?;
            let from = length - index - 1;
            let value = self.typed_array_read_number(record, from)?;
            // The source is never written. A resource refusal can leave only
            // complete scalar stores in the private, unreturned destination.
            self.typed_array_write_number(target, index, value)?;
            index += 1;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
