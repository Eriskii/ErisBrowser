//! Non-shared views with live backing witnesses and bounded scalar byte access.
use super::array_buffer::BufferId;
use super::scalar_codec::Codec;
#[cfg(test)]
use super::scalar_codec::{decode_f16, encode_f16};
use super::*;

mod bootstrap;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug)]
enum ViewLength {
    Fixed(usize),
    Tracking,
}

#[derive(Clone, Copy, Debug)]
struct Record {
    object_id: usize,
    buffer: BufferId,
    offset: usize,
    length: ViewLength,
}

#[derive(Default)]
pub(super) struct State {
    // No callback follows ordinary object creation and record publication.
    // These sorted, append-only indices and copied BufferIds survive callbacks.
    records: Vec<Record>,
    prototype: Option<usize>,
}

impl Runtime {
    fn data_view_find(&mut self, value: &Value) -> Result<Option<usize>> {
        let Value::Object(id) = value else {
            return Ok(None);
        };
        let mut first = 0;
        let mut last = self.data_views.records.len();
        while first < last {
            self.tick()?;
            let middle = first + (last - first) / 2;
            match self.data_views.records[middle].object_id.cmp(id) {
                Ordering::Less => first = middle + 1,
                Ordering::Greater => last = middle,
                Ordering::Equal => return Ok(Some(middle)),
            }
        }
        Ok(None)
    }

    pub(super) fn data_view_is_view(&mut self, value: &Value) -> Result<bool> {
        // Empty state performs no search: ArrayBuffer's existing dispatch tick
        // covers this predicate, including the original cheap no-view path.
        Ok(self.data_view_find(value)?.is_some())
    }

    fn data_view_record(&mut self, value: &Value) -> Result<Record> {
        self.data_view_find(value)?
            .map(|index| self.data_views.records[index])
            .ok_or_else(|| ScriptError::type_error("receiver is not a DataView"))
    }

    fn data_view_reserve_record(&mut self) -> Result<()> {
        let records = &self.data_views.records;
        if records.len() < records.capacity() {
            return self.tick();
        }
        let capacity = records
            .capacity()
            .checked_mul(2)
            .map(|n| n.max(4))
            .ok_or_else(|| ScriptError::resource("DataView record capacity overflow"))?;
        let bytes = capacity
            .checked_mul(std::mem::size_of::<Record>())
            .ok_or_else(|| ScriptError::resource("DataView record allocation overflow"))?;
        let moved = records
            .len()
            .checked_mul(std::mem::size_of::<Record>())
            .ok_or_else(|| ScriptError::resource("DataView record movement overflow"))?;
        let additional = capacity - records.len();
        self.work(1 + moved.div_ceil(8).saturating_mul(2))?;
        // Full requested block and logical old/new overlap, not a measurement
        // of allocator rounding/overhead (reserve_exact may grant more).
        self.charge(bytes)?;
        self.data_views
            .records
            .try_reserve_exact(additional)
            .map_err(|_| ScriptError::resource("DataView record allocation failed"))
    }

    fn data_view_length(&mut self, record: Record) -> Result<usize> {
        let metadata = self.buffer_view_metadata(record.buffer)?;
        let length = metadata
            .byte_length
            .ok_or_else(|| ScriptError::type_error("DataView buffer is detached"))?;
        let remaining = length
            .checked_sub(record.offset)
            .ok_or_else(|| ScriptError::type_error("DataView is out of bounds"))?;
        match record.length {
            ViewLength::Tracking => Ok(remaining),
            ViewLength::Fixed(length) if length <= remaining => Ok(length),
            ViewLength::Fixed(_) => Err(ScriptError::type_error("DataView is out of bounds")),
        }
    }

    pub(super) fn data_view_constructor(
        &mut self,
        arguments: &[Value],
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let buffer = self.buffer_for_view(arguments.first().unwrap_or(&Value::Undefined))?;
        let offset =
            self.buffer_index(arguments.get(1).cloned().unwrap_or(Value::Undefined), doc)?;
        let metadata = self.buffer_view_metadata(buffer)?;
        let captured = metadata
            .byte_length
            .ok_or_else(|| ScriptError::type_error("DataView buffer is detached"))?
            as u64;
        if offset > captured {
            return Err(ScriptError::range_error("DataView offset exceeds buffer"));
        }
        let explicit = arguments
            .get(2)
            .filter(|value| !matches!(value, Value::Undefined));
        let length = if let Some(value) = explicit {
            let length = self.buffer_index(value.clone(), doc)?;
            if length > captured - offset {
                return Err(ScriptError::range_error("DataView length exceeds buffer"));
            }
            ViewLength::Fixed(length as usize)
        } else if metadata.resizable {
            ViewLength::Tracking
        } else {
            ViewLength::Fixed((captured - offset) as usize)
        };
        // Explicit length conversion used captured bounds. Its detach/resize
        // effects are checked only after prototype lookup and object creation.
        let prototype = self.splice_named_get(&new_target, "prototype", doc)?;
        let prototype = if js_object(&prototype) {
            prototype
        } else {
            Value::Object(self.data_views.prototype.expect("DataView initialized"))
        };
        let object = self.object_ordered([])?;
        let Value::Object(object_id) = object else {
            unreachable!()
        };
        self.objects[object_id].prototype = Some(prototype);
        let current = self
            .buffer_view_metadata(buffer)?
            .byte_length
            .ok_or_else(|| ScriptError::type_error("DataView buffer is detached"))?
            as u64;
        if offset > current {
            return Err(ScriptError::range_error("DataView offset exceeds buffer"));
        }
        if explicit.is_some()
            && let ViewLength::Fixed(length) = length
            && length as u64 > current - offset
        {
            return Err(ScriptError::range_error("DataView length exceeds buffer"));
        }
        self.data_view_reserve_record()?;
        debug_assert!(
            self.data_views
                .records
                .last()
                .is_none_or(|record| record.object_id < object_id)
        );
        self.data_views.records.push(Record {
            object_id,
            buffer,
            offset: offset as usize,
            length,
        });
        Ok(object)
    }

    pub(super) fn data_view_native(
        &mut self,
        method: &str,
        receiver: Value,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let record = self.data_view_record(&receiver)?;
        if method == "getBuffer" {
            return self.buffer_view_value(record.buffer);
        }
        if matches!(method, "getByteLength" | "getByteOffset") {
            let length = self.data_view_length(record)?;
            return Ok(Value::Number(if method == "getByteLength" {
                length
            } else {
                record.offset
            } as f64));
        }
        let (write, suffix) = if let Some(suffix) = method.strip_prefix("get") {
            (false, suffix)
        } else if let Some(suffix) = method.strip_prefix("set") {
            (true, suffix)
        } else {
            return Err(ScriptError::type_error("unknown DataView method"));
        };
        let codec = Codec::named(suffix)
            .ok_or_else(|| ScriptError::type_error("unknown DataView method"))?;
        let index =
            self.buffer_index(arguments.first().cloned().unwrap_or(Value::Undefined), doc)?;
        let value = if write {
            self.splice_number(arguments.get(1).cloned().unwrap_or(Value::Undefined), doc)?
        } else {
            0.0
        };
        let little = arguments
            .get(if write { 2 } else { 1 })
            .is_some_and(Value::truthy);
        let length = self.data_view_length(record)?;
        let width = codec.width();
        if index
            .checked_add(width as u64)
            .is_none_or(|end| end > length as u64)
        {
            return Err(ScriptError::range_error("DataView element exceeds view"));
        }
        let offset = record
            .offset
            .checked_add(index as usize)
            .ok_or_else(|| ScriptError::range_error("DataView element offset overflow"))?;
        self.work(codec.work())?;
        if write {
            let bytes = codec.encode(value, little);
            self.buffer_view_write(record.buffer, offset, &bytes[..width])?;
            Ok(Value::Undefined)
        } else {
            let bytes = self.buffer_view_read(record.buffer, offset, width)?;
            Ok(Value::Number(codec.decode(bytes, little)))
        }
    }
}
