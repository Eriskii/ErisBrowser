//! Number TypedArray fill with ordered conversion and fresh final bounds.
use super::*;

impl Runtime {
    fn typed_array_fill_index(
        &mut self,
        value: Value,
        length: usize,
        doc: &mut Document,
    ) -> Result<usize> {
        let number = self.splice_number(value, doc)?;
        // Conversion callbacks precede admission of normalization/clamping.
        self.work(4)?;
        let number = integer_or_infinity(number);
        Ok(if number < 0.0 {
            (length as f64 + number).max(0.0) as usize
        } else {
            number.min(length as f64) as usize
        })
    }

    pub(super) fn typed_array_fill(
        &mut self,
        receiver: Value,
        record: Record,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8)?;
        let length = self.typed_array_length(record)?;
        self.work(4)?;
        // Convert the value once, including for an initially empty view. The
        // later scalar encoding cannot invoke author code.
        let value =
            self.splice_number(arguments.first().cloned().unwrap_or(Value::Undefined), doc)?;
        self.work(4)?;
        let first = self.typed_array_fill_index(
            arguments.get(1).cloned().unwrap_or(Value::Undefined),
            length,
            doc,
        )?;
        self.work(4)?;
        let end = arguments.get(2).cloned().unwrap_or(Value::Undefined);
        let last = if matches!(end, Value::Undefined) {
            length
        } else {
            self.typed_array_fill_index(end, length, doc)?
        };
        // Validate only after every conversion, and do so even for an empty
        // requested range. A callback can revive an intermediate OOB view.
        let live = self
            .typed_array_live(record)?
            .ok_or_else(|| ScriptError::type_error("TypedArray is detached or out of bounds"))?;
        self.work(8)?;
        let last = last.min(live.length);
        let mut index = first;
        if index >= last {
            return Ok(receiver);
        }
        // Pay for retaining the final witness/width, codec/clamp selection,
        // and the encoded scalar and source-slice bindings before preparing
        // them. Empty ranges keep their original conversion/validation fee.
        self.work(8)?;
        let width = record.kind.width();
        let codec = record.kind.codec();
        let value = if record.kind == Kind::Uint8Clamped {
            self.work(16)?;
            clamp_u8(value)
        } else {
            value
        };
        self.work(codec.work())?;
        let bytes = codec.encode(value, true);
        let bytes = &bytes[..width];
        while index < last {
            // No callbacks or shared backing can invalidate the copied final
            // witness. Keep checked offsets and the existing buffer writer,
            // which prepays each complete scalar before its first byte write.
            self.work(8)?;
            let offset = self.typed_array_offset(live, record.kind, index)?;
            self.buffer_view_write(live.buffer, offset, bytes)?;
            // Backing admission bounds every index below MAX_HEAP and usize
            // overflow. No callback or shared-buffer mutation occurs here.
            index += 1;
        }
        Ok(receiver)
    }
}

#[cfg(test)]
mod tests;
