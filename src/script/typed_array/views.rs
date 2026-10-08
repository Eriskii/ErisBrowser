//! Shared-buffer views and Number-only joined text with ordered callbacks.
use super::*;

impl Runtime {
    fn typed_array_view_index(
        &mut self,
        value: Value,
        length: usize,
        doc: &mut Document,
    ) -> Result<usize> {
        let number = self.splice_number(value, doc)?;
        self.work(4)?;
        let number = integer_or_infinity(number);
        // Captured lengths are bounded by admitted non-shared backing storage.
        Ok(if number < 0.0 {
            (length as f64 + number).max(0.0) as usize
        } else {
            number.min(length as f64) as usize
        })
    }

    fn typed_array_view_species(
        &mut self,
        receiver: &Value,
        kind: Kind,
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8)?;
        let constructor = self.splice_named_get(receiver, "constructor", doc)?;
        if matches!(constructor, Value::Undefined) {
            return Ok(self.typed_arrays.constructors[kind.index()]
                .as_ref()
                .unwrap()
                .clone());
        }
        if !js_object(&constructor) {
            return Err(ScriptError::type_error(
                "TypedArray constructor is not an object",
            ));
        }
        let species = self.splice_symbol_get(&constructor, "species", doc)?;
        if matches!(species, Value::Null | Value::Undefined) {
            return Ok(self.typed_arrays.constructors[kind.index()]
                .as_ref()
                .unwrap()
                .clone());
        }
        if !self.is_constructor(species.clone())? {
            return Err(ScriptError::type_error(
                "TypedArray species is not a constructor",
            ));
        }
        Ok(species)
    }

    pub(super) fn typed_array_subarray(
        &mut self,
        receiver: Value,
        record: Record,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(12)?;
        let backing = record
            .backing
            .ok_or_else(|| ScriptError::type_error("uninitialized TypedArray"))?;
        // Subarray deliberately allows an initially detached/OOB source. Keep
        // its stored offset and capture length zero before coercion can revive it.
        let length = self.typed_array_live(record)?.map_or(0, |live| live.length);
        let first = self.typed_array_view_index(
            arguments.first().cloned().unwrap_or(Value::Undefined),
            length,
            doc,
        )?;
        self.work(8)?;
        let end = arguments.get(1).cloned().unwrap_or(Value::Undefined);
        let tracking =
            matches!(backing.length, ViewLength::Tracking) && matches!(end, Value::Undefined);
        let last = if matches!(end, Value::Undefined) {
            length
        } else {
            self.typed_array_view_index(end, length, doc)?
        };
        self.work(12)?;
        let offset = first
            .checked_mul(record.kind.width())
            .and_then(|bytes| backing.offset.checked_add(bytes))
            .ok_or_else(|| ScriptError::resource("TypedArray subarray offset overflow"))?;
        let count = last.saturating_sub(first);
        let arity = if tracking { 2 } else { 3 };
        let constructor = self.typed_array_view_species(&receiver, record.kind, doc)?;
        self.work(8 + 2 * arity)?;
        self.charge(arity * std::mem::size_of::<Value>())?;
        let mut parameters = Vec::new();
        parameters
            .try_reserve_exact(arity)
            .map_err(|_| ScriptError::resource("TypedArray species argument allocation failed"))?;
        parameters.push(self.buffer_view_value(backing.buffer)?);
        parameters.push(Value::Number(offset as f64));
        if !tracking {
            parameters.push(Value::Number(count as f64));
        }
        let result =
            self.construct_with_target(constructor.clone(), parameters, constructor, doc)?;
        self.work(8)?;
        let result_record = self.typed_array_record(&result)?.ok_or_else(|| {
            ScriptError::type_error("TypedArray species returned a non-TypedArray")
        })?;
        self.typed_array_length(result_record)?;
        // All implemented kinds have Number content. Two/three-argument
        // species construction has no minimum-length or source-identity test.
        Ok(result)
    }

    pub(super) fn typed_array_join(
        &mut self,
        record: Record,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8)?;
        let length = self.typed_array_length(record)?;
        self.work(4)?;
        let separator = match arguments.first() {
            None | Some(Value::Undefined) => {
                self.charge(32)?;
                JsString::from(",")
            }
            Some(value) => self.string_hint(value.clone(), doc)?,
        };
        let mut output = Vec::new();
        for index in 0..length {
            // Pay every captured position even when its current value and the
            // separator are empty. The owned record survives callback growth.
            self.work(6)?;
            if index != 0 {
                self.array_join_append(&mut output, &separator)?;
            }
            let Some(live) = self
                .typed_array_live(record)?
                .filter(|live| index < live.length)
            else {
                continue;
            };
            // No author callback occurs between this fresh witness and read.
            // An absent index never consults a prototype or poisoned length.
            let offset = self.typed_array_offset(live, record.kind, index)?;
            let bytes = self.buffer_view_read(live.buffer, offset, record.kind.width())?;
            self.work(record.kind.codec().work())?;
            let number = record.kind.codec().decode(bytes, true);
            let text = self.string_hint(Value::Number(number), doc)?;
            self.array_join_append(&mut output, &text)?;
        }
        // Vec-to-Rc conversion copies the complete UTF-16 payload. Admission
        // precedes that final allocation; no generic Array cycle state is used.
        self.work(4 + output.len().div_ceil(8))?;
        self.charge(
            output
                .len()
                .checked_mul(2)
                .and_then(|n| n.checked_add(24))
                .ok_or_else(|| {
                    ScriptError::resource("TypedArray join result allocation overflow")
                })?,
        )?;
        Ok(Value::String(output.into()))
    }
}

#[cfg(test)]
mod tests;
