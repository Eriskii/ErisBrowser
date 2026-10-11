//! Number TypedArray copyWithin with ordered coercion and raw overlapping copies.
use super::*;

impl Runtime {
    fn typed_array_copy_within_index(
        &mut self,
        value: Value,
        length: usize,
        doc: &mut Document,
    ) -> Result<usize> {
        let number = self.splice_number(value, doc)?;
        self.work(4)?;
        let number = integer_or_infinity(number);
        Ok(if number < 0.0 {
            (length as f64 + number).max(0.0) as usize
        } else {
            number.min(length as f64) as usize
        })
    }

    pub(super) fn typed_array_copy_within(
        &mut self,
        receiver: Value,
        record: Record,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8)?;
        let length = self.typed_array_length(record)?;
        self.work(4)?;
        let target = self.typed_array_copy_within_index(
            arguments.first().cloned().unwrap_or(Value::Undefined),
            length,
            doc,
        )?;
        self.work(4)?;
        let start = self.typed_array_copy_within_index(
            arguments.get(1).cloned().unwrap_or(Value::Undefined),
            length,
            doc,
        )?;
        self.work(4)?;
        let end = arguments.get(2).cloned().unwrap_or(Value::Undefined);
        let end = if matches!(end, Value::Undefined) {
            length
        } else {
            self.typed_array_copy_within_index(end, length, doc)?
        };
        self.work(8)?;
        let count = end.saturating_sub(start).min(length - target);
        if count == 0 {
            // All conversions occur, but an initially empty copy does not
            // revalidate backing that those conversions detached or resized.
            return Ok(receiver);
        }
        let live = self
            .typed_array_live(record)?
            .ok_or_else(|| ScriptError::type_error("TypedArray is detached or out of bounds"))?;
        self.work(8)?;
        let count = count
            .min(live.length.saturating_sub(start))
            .min(live.length.saturating_sub(target));
        if count == 0 {
            return Ok(receiver);
        }
        self.work(8)?;
        let width = record.kind.width();
        // The clipped source end is bounded by live.length. Overlap direction
        // in elements is equivalent to direction in bytes for this one view.
        let backward = start < target && target < start + count;
        let mut remaining = count;
        while remaining > 0 {
            self.work(8)?;
            let delta = if backward {
                remaining - 1
            } else {
                count - remaining
            };
            let from = self.typed_array_offset(live, record.kind, start + delta)?;
            let to = self.typed_array_offset(live, record.kind, target + delta)?;
            // No callback/shared mutation follows the final witness. Copy a
            // complete raw element before storing; never decode NaN payloads.
            let bytes = self.buffer_view_read(live.buffer, from, width)?;
            self.buffer_view_write(live.buffer, to, &bytes[..width])?;
            remaining -= 1;
        }
        Ok(receiver)
    }
}

#[cfg(test)]
mod quota_tests;
#[cfg(test)]
mod tests;
