//! Number TypedArray reverse using the existing admitted scalar operations.
use super::*;

impl Runtime {
    pub(super) fn typed_array_reverse(&mut self, receiver: Value, record: Record) -> Result<Value> {
        self.work(8)?;
        // Empty and one-element views still reject detached or OOB backing.
        // No argument conversion, property access or author callback follows.
        let length = self.typed_array_length(record)?;
        self.work(4)?;
        let middle = length / 2;
        let mut lower = 0;
        while lower < middle {
            // Admit pair selection and the eventual lower-index advance.
            // lower < floor(length/2) proves both subtraction bounds.
            self.work(8)?;
            let upper = length - lower - 1;
            let lower_value = self.typed_array_read_number(record, lower)?;
            let upper_value = self.typed_array_read_number(record, upper)?;
            // Read both values before either store. Each unchanged writer
            // prepays a whole scalar; a later refusal can retain the lower
            // store without the upper store, but never a partial scalar.
            self.typed_array_write_number(record, lower, upper_value)?;
            self.typed_array_write_number(record, upper, lower_value)?;
            lower += 1;
        }
        Ok(receiver)
    }
}

#[cfg(test)]
mod tests;
