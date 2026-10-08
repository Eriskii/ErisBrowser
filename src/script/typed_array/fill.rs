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
        // scalar writer receives this Number and cannot invoke author code.
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
        let current_length = self.typed_array_length(record)?;
        self.work(8)?;
        let last = last.min(current_length);
        let mut index = first;
        while index < last {
            // Pay each reached position/advance before the existing scalar
            // writer, whose entire byte copy is prepaid. A later refusal may
            // leave completed elements written, but never a partial scalar.
            self.work(8)?;
            self.typed_array_write_number(record, index, value)?;
            // Backing admission bounds every index below MAX_HEAP and usize
            // overflow. No callback or shared-buffer mutation occurs here.
            index += 1;
        }
        Ok(receiver)
    }
}

#[cfg(test)]
mod tests;
