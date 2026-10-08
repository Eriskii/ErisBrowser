//! Non-shared ArrayBuffer slots, ordered callbacks and transactional byte storage.
use super::*;

mod bootstrap;

#[cfg(test)]
mod tests;

const MAX_INDEX: u64 = 9_007_199_254_740_991;

// Only this module can construct or inspect backing-table indices. A copied
// handle survives callbacks, record-table growth, resize and detachment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BufferId(usize);

#[derive(Clone, Copy)]
pub(super) struct ViewBufferMetadata {
    pub(super) byte_length: Option<usize>,
    pub(super) resizable: bool,
}

struct Record {
    object_id: usize,
    // Some(empty) is attached. Detachment preserves max_byte_length.
    bytes: Option<Vec<u8>>,
    max_byte_length: Option<u64>,
}

#[derive(Default)]
pub(super) struct State {
    // Object ids increase monotonically, and insertion has no author callback.
    // Append-only indices survive callbacks; Rust references to records do not.
    records: Vec<Record>,
    intrinsic: Option<Value>,
    prototype: Option<usize>,
}

fn tree_bound(count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let mut capacity = count.saturating_add(1) / 2;
    let mut nodes = 1usize;
    while capacity >= 6 {
        capacity /= 6;
        nodes += 1;
    }
    (count.min(11usize.saturating_mul(nodes)), nodes)
}

fn search_work(count: usize, units: usize) -> usize {
    tree_bound(count).0.saturating_mul(1 + units / 8)
}

fn byte_work(length: usize) -> usize {
    if length == 0 {
        0
    } else {
        1 + length.div_ceil(8)
    }
}

impl Runtime {
    fn buffer_record(&mut self, value: &Value) -> Result<usize> {
        self.tick()?;
        let Value::Object(id) = value else {
            return Err(ScriptError::type_error("receiver is not an ArrayBuffer"));
        };
        let mut first = 0;
        let mut last = self.array_buffers.records.len();
        while first < last {
            self.tick()?;
            let middle = first + (last - first) / 2;
            match self.array_buffers.records[middle].object_id.cmp(id) {
                Ordering::Less => first = middle + 1,
                Ordering::Greater => last = middle,
                Ordering::Equal => return Ok(middle),
            }
        }
        Err(ScriptError::type_error("receiver is not an ArrayBuffer"))
    }

    fn buffer_attached_length(&self, index: usize) -> Result<usize> {
        self.array_buffers.records[index]
            .bytes
            .as_ref()
            .map(Vec::len)
            .ok_or_else(|| ScriptError::type_error("ArrayBuffer is detached"))
    }

    pub(super) fn buffer_index(&mut self, value: Value, doc: &mut Document) -> Result<u64> {
        let number = integer_or_infinity(self.splice_number(value, doc)?);
        if !(0.0..=MAX_INDEX as f64).contains(&number) {
            return Err(ScriptError::range_error("invalid ArrayBuffer length"));
        }
        Ok(number as u64)
    }

    fn buffer_clamped_index(
        &mut self,
        value: Value,
        length: usize,
        doc: &mut Document,
    ) -> Result<usize> {
        let number = integer_or_infinity(self.splice_number(value, doc)?);
        Ok(if number < 0.0 {
            (length as f64 + number).max(0.0) as usize
        } else {
            number.min(length as f64) as usize
        })
    }

    fn buffer_reserve_record(&mut self) -> Result<()> {
        let records = &self.array_buffers.records;
        if records.len() < records.capacity() {
            return self.tick();
        }
        let capacity = records
            .capacity()
            .checked_mul(2)
            .map(|n| n.max(4))
            .ok_or_else(|| ScriptError::resource("ArrayBuffer record capacity overflow"))?;
        let bytes = capacity
            .checked_mul(std::mem::size_of::<Record>())
            .ok_or_else(|| ScriptError::resource("ArrayBuffer record allocation overflow"))?;
        let moved = records.len().saturating_mul(std::mem::size_of::<Record>());
        let additional = capacity - records.len();
        self.work(1 + byte_work(moved).saturating_mul(2))?;
        self.charge(bytes)?;
        self.array_buffers
            .records
            .try_reserve_exact(additional)
            .map_err(|_| ScriptError::resource("ArrayBuffer record allocation failed"))
    }

    fn buffer_empty_block(&mut self, length: u64) -> Result<Vec<u8>> {
        // This absolute host capacity is independent of remaining VM budget.
        // CreateByteDataBlock rejects an impossible block before zeroing; the
        // caller has already performed prototype lookup/object creation.
        if length > MAX_HEAP as u64 {
            return Err(ScriptError::range_error(
                "ArrayBuffer length exceeds host capacity",
            ));
        }
        let length = usize::try_from(length)
            .map_err(|_| ScriptError::resource("ArrayBuffer allocation length overflow"))?;
        self.work(byte_work(length))?;
        self.charge(length)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::range_error("ArrayBuffer backing allocation failed"))?;
        bytes.resize(length, 0);
        Ok(bytes)
    }

    fn buffer_allocate(
        &mut self,
        length: u64,
        maximum: Option<u64>,
        new_target: Value,
        doc: &mut Document,
    ) -> Result<(Value, usize)> {
        self.tick()?;
        if maximum.is_some_and(|maximum| length > maximum) {
            return Err(ScriptError::range_error(
                "ArrayBuffer length exceeds maximum",
            ));
        }
        let prototype = self.splice_named_get(&new_target, "prototype", doc)?;
        let prototype = if js_object(&prototype) {
            prototype
        } else {
            Value::Object(
                self.array_buffers
                    .prototype
                    .expect("ArrayBuffer initialized"),
            )
        };
        // OrdinaryCreateFromConstructor precedes backing allocation and the
        // late maximum feasibility check. No object escapes on failure.
        self.work(search_work(self.prototypes.len(), 6))?;
        let object = self.object_ordered([])?;
        let Value::Object(object_id) = object else {
            unreachable!()
        };
        self.objects[object_id].prototype = Some(prototype);
        let bytes = self.buffer_empty_block(length)?;
        // Host feasibility policy: reuse the absolute VM heap ceiling, without
        // reserving or charging unallocated maximum bytes. Remaining budget
        // failures are terminal resources, not this semantic RangeError.
        if maximum.is_some_and(|maximum| maximum > MAX_HEAP as u64) {
            return Err(ScriptError::range_error(
                "ArrayBuffer maximum exceeds host capacity",
            ));
        }
        self.buffer_reserve_record()?;
        let index = self.array_buffers.records.len();
        debug_assert!(
            self.array_buffers
                .records
                .last()
                .is_none_or(|r| r.object_id < object_id)
        );
        self.array_buffers.records.push(Record {
            object_id,
            bytes: Some(bytes),
            max_byte_length: maximum,
        });
        Ok((object, index))
    }

    pub(super) fn array_buffer_constructor(
        &mut self,
        arguments: &[Value],
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let length =
            self.buffer_index(arguments.first().cloned().unwrap_or(Value::Undefined), doc)?;
        let options = arguments.get(1).cloned().unwrap_or(Value::Undefined);
        let maximum = if js_object(&options) {
            let value = self.splice_named_get(&options, "maxByteLength", doc)?;
            if matches!(value, Value::Undefined) {
                None
            } else {
                Some(self.buffer_index(value, doc)?)
            }
        } else {
            None
        };
        self.buffer_allocate(length, maximum, new_target, doc)
            .map(|(value, _)| value)
    }

    fn buffer_resize(&mut self, index: usize, value: Value, doc: &mut Document) -> Result<Value> {
        // A detached RAB still has this slot; a fixed buffer never does.
        let maximum = self.array_buffers.records[index]
            .max_byte_length
            .ok_or_else(|| ScriptError::type_error("ArrayBuffer is not resizable"))?;
        let length = self.buffer_index(value, doc)?;
        let previous = self.buffer_attached_length(index)?;
        if length > maximum {
            return Err(ScriptError::range_error(
                "ArrayBuffer resize exceeds maximum",
            ));
        }
        let length = usize::try_from(length)
            .map_err(|_| ScriptError::resource("ArrayBuffer resize length overflow"))?;
        self.tick()?;
        if length <= previous {
            self.array_buffers.records[index]
                .bytes
                .as_mut()
                .unwrap()
                .truncate(length);
        } else {
            self.work(byte_work(length - previous))?;
            let capacity = self.array_buffers.records[index]
                .bytes
                .as_ref()
                .unwrap()
                .capacity();
            if length <= capacity {
                self.array_buffers.records[index]
                    .bytes
                    .as_mut()
                    .unwrap()
                    .resize(length, 0);
            } else {
                self.work(byte_work(previous).saturating_mul(2))?;
                self.charge(length)?;
                let mut replacement = Vec::new();
                replacement.try_reserve_exact(length).map_err(|_| {
                    ScriptError::range_error("ArrayBuffer resize allocation failed")
                })?;
                replacement
                    .extend_from_slice(self.array_buffers.records[index].bytes.as_ref().unwrap());
                replacement.resize(length, 0);
                self.array_buffers.records[index].bytes = Some(replacement);
            }
        }
        Ok(Value::Undefined)
    }

    fn buffer_species(&mut self, receiver: &Value, doc: &mut Document) -> Result<Value> {
        let constructor = self.splice_named_get(receiver, "constructor", doc)?;
        if matches!(constructor, Value::Undefined) {
            return Ok(self.array_buffers.intrinsic.as_ref().unwrap().clone());
        }
        if !js_object(&constructor) {
            return Err(ScriptError::type_error(
                "ArrayBuffer constructor is not an object",
            ));
        }
        let species = self.splice_symbol_get(&constructor, "species", doc)?;
        if matches!(species, Value::Null | Value::Undefined) {
            return Ok(self.array_buffers.intrinsic.as_ref().unwrap().clone());
        }
        if !self.is_constructor(species.clone())? {
            return Err(ScriptError::type_error(
                "ArrayBuffer species is not a constructor",
            ));
        }
        Ok(species)
    }

    // No callbacks or fallible steps occur after this function is entered.
    fn buffer_copy(&mut self, source: usize, target: usize, first: usize, count: usize) {
        debug_assert_ne!(source, target);
        let (from, to) = if source < target {
            let (before, after) = self.array_buffers.records.split_at_mut(target);
            (&before[source], &mut after[0])
        } else {
            let (before, after) = self.array_buffers.records.split_at_mut(source);
            (&after[0], &mut before[target])
        };
        let bytes = from.bytes.as_ref().unwrap();
        to.bytes.as_mut().unwrap()[..count].copy_from_slice(&bytes[first..first + count]);
    }

    fn buffer_slice(
        &mut self,
        receiver: Value,
        index: usize,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let length = self.buffer_attached_length(index)?;
        let first = self.buffer_clamped_index(
            arguments.first().cloned().unwrap_or(Value::Undefined),
            length,
            doc,
        )?;
        let end = arguments.get(1).cloned().unwrap_or(Value::Undefined);
        let last = if matches!(end, Value::Undefined) {
            length
        } else {
            self.buffer_clamped_index(end, length, doc)?
        };
        let requested = last.saturating_sub(first);
        let constructor = self.buffer_species(&receiver, doc)?;
        self.tick()?;
        self.charge(std::mem::size_of::<Value>())?;
        let mut args = Vec::new();
        args.try_reserve_exact(1)
            .map_err(|_| ScriptError::resource("ArrayBuffer species argument allocation failed"))?;
        args.push(Value::Number(requested as f64));
        let result = self.construct_with_target(constructor.clone(), args, constructor, doc)?;
        let target = self.buffer_record(&result)?;
        let capacity = self.buffer_attached_length(target)?;
        if index == target {
            return Err(ScriptError::type_error(
                "ArrayBuffer species returned its source",
            ));
        }
        if capacity < requested {
            return Err(ScriptError::type_error(
                "ArrayBuffer species result is too small",
            ));
        }
        let current = self.buffer_attached_length(index)?;
        let count = requested.min(current.saturating_sub(first));
        self.work(byte_work(count).saturating_mul(2))?;
        // When the source shrank past first there is no valid source range,
        // even for a zero-byte copy. Existing destination suffix is untouched.
        if count != 0 {
            self.buffer_copy(index, target, first, count);
        }
        Ok(result)
    }

    fn buffer_transfer(
        &mut self,
        index: usize,
        value: Value,
        preserve: bool,
        doc: &mut Document,
    ) -> Result<Value> {
        let length = if matches!(value, Value::Undefined) {
            self.array_buffers.records[index]
                .bytes
                .as_ref()
                .map_or(0, Vec::len) as u64
        } else {
            self.buffer_index(value, doc)?
        };
        let current = self.buffer_attached_length(index)?;
        let maximum = if preserve {
            self.array_buffers.records[index].max_byte_length
        } else {
            None
        };
        // [[ArrayBufferDetachKey]] is always undefined here: there is no host
        // detachment API or key-bearing buffer producer in this runtime.
        let intrinsic = self.array_buffers.intrinsic.as_ref().unwrap().clone();
        let (result, target) = self.buffer_allocate(length, maximum, intrinsic, doc)?;
        let count = current.min(
            self.array_buffers.records[target]
                .bytes
                .as_ref()
                .unwrap()
                .len(),
        );
        self.work(byte_work(count).saturating_mul(2))?;
        self.buffer_copy(index, target, 0, count);
        self.array_buffers.records[index].bytes = None;
        // No fallible operation, callback or allocation may follow detachment.
        Ok(result)
    }

    pub(super) fn buffer_for_view(&mut self, value: &Value) -> Result<BufferId> {
        self.buffer_record(value).map(BufferId)
    }

    pub(super) fn buffer_view_metadata(&mut self, id: BufferId) -> Result<ViewBufferMetadata> {
        self.tick()?;
        let record = &self.array_buffers.records[id.0];
        Ok(ViewBufferMetadata {
            byte_length: record.bytes.as_ref().map(Vec::len),
            resizable: record.max_byte_length.is_some(),
        })
    }

    pub(super) fn buffer_view_value(&mut self, id: BufferId) -> Result<Value> {
        self.tick()?;
        Ok(Value::Object(self.array_buffers.records[id.0].object_id))
    }

    pub(super) fn buffer_view_read(
        &mut self,
        id: BufferId,
        offset: usize,
        length: usize,
    ) -> Result<[u8; 8]> {
        self.tick()?;
        if !(1..=8).contains(&length) {
            return Err(ScriptError::range_error("invalid view element width"));
        }
        let attached = self.buffer_attached_length(id.0)?;
        let end = offset
            .checked_add(length)
            .filter(|end| *end <= attached)
            .ok_or_else(|| ScriptError::range_error("view byte range exceeds buffer"))?;
        self.work(8 + length)?;
        let mut result = [0; 8];
        result[..length].copy_from_slice(
            &self.array_buffers.records[id.0].bytes.as_ref().unwrap()[offset..end],
        );
        Ok(result)
    }

    pub(super) fn buffer_view_write(
        &mut self,
        id: BufferId,
        offset: usize,
        bytes: &[u8],
    ) -> Result<()> {
        self.tick()?;
        if !(1..=8).contains(&bytes.len()) {
            return Err(ScriptError::range_error("invalid view element width"));
        }
        let attached = self.buffer_attached_length(id.0)?;
        let end = offset
            .checked_add(bytes.len())
            .filter(|end| *end <= attached)
            .ok_or_else(|| ScriptError::range_error("view byte range exceeds buffer"))?;
        // The entire mutation is prepaid. No callback or fallible operation
        // follows the first byte write, including for unaligned elements.
        self.work(bytes.len())?;
        self.array_buffers.records[id.0].bytes.as_mut().unwrap()[offset..end]
            .copy_from_slice(bytes);
        Ok(())
    }

    fn array_buffer_is_view(&mut self, value: &Value) -> Result<bool> {
        Ok(self.data_view_is_view(value)? || self.typed_array_is_view(value)?)
    }

    pub(super) fn array_buffer_native(
        &mut self,
        method: &str,
        receiver: Value,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        if method == "isView" {
            return Ok(Value::Bool(self.array_buffer_is_view(
                arguments.first().unwrap_or(&Value::Undefined),
            )?));
        }
        if method == "species" {
            return Ok(receiver);
        }
        let index = self.buffer_record(&receiver)?;
        let record = &self.array_buffers.records[index];
        match method {
            "getByteLength" => Ok(Value::Number(
                record.bytes.as_ref().map_or(0, Vec::len) as f64
            )),
            "getMaxByteLength" => Ok(Value::Number(record.bytes.as_ref().map_or(0, |bytes| {
                record.max_byte_length.unwrap_or(bytes.len() as u64)
            }) as f64)),
            "getResizable" => Ok(Value::Bool(record.max_byte_length.is_some())),
            "getDetached" => Ok(Value::Bool(record.bytes.is_none())),
            "resize" => self.buffer_resize(
                index,
                arguments.first().cloned().unwrap_or(Value::Undefined),
                doc,
            ),
            "slice" => self.buffer_slice(receiver, index, arguments, doc),
            "transfer" | "transferToFixedLength" => self.buffer_transfer(
                index,
                arguments.first().cloned().unwrap_or(Value::Undefined),
                method == "transfer",
                doc,
            ),
            _ => Err(ScriptError::unsupported(
                "ArrayBuffer operation is not implemented",
            )),
        }
    }
}

// Draft addition within the existing array_buffer.rs module. Existing public
// ArrayBuffer/DataView operations and their fees are not changed by this text.
impl Runtime {
    pub(super) fn buffer_probe(&mut self, value: &Value) -> Result<Option<BufferId>> {
        self.tick()?;
        let Value::Object(id) = value else {
            return Ok(None);
        };
        let mut first = 0;
        let mut last = self.array_buffers.records.len();
        while first < last {
            self.tick()?;
            let middle = first + (last - first) / 2;
            match self.array_buffers.records[middle].object_id.cmp(id) {
                Ordering::Less => first = middle + 1,
                Ordering::Greater => last = middle,
                Ordering::Equal => return Ok(Some(BufferId(middle))),
            }
        }
        Ok(None)
    }

    pub(super) fn buffer_allocate_intrinsic(&mut self, length: u64) -> Result<BufferId> {
        self.tick()?;
        let prototype = self
            .array_buffers
            .prototype
            .expect("ArrayBuffer initialized");
        // object_ordered creates an ordinary bag using the fixed Object
        // prototype. Retain the existing buffer allocator's lookup allowance.
        self.work(search_work(self.prototypes.len(), 6))?;
        let object = self.object_ordered([])?;
        let Value::Object(object_id) = object else {
            unreachable!()
        };
        self.objects[object_id].prototype = Some(Value::Object(prototype));
        // The existing helper pays zeroing, cumulative bytes, and applies the
        // same absolute host-capacity RangeError policy before host allocation.
        let bytes = self.buffer_empty_block(length)?;
        self.buffer_reserve_record()?;
        self.work(4)?;
        let index = self.array_buffers.records.len();
        debug_assert!(
            self.array_buffers
                .records
                .last()
                .is_none_or(|r| r.object_id < object_id)
        );
        self.array_buffers.records.push(Record {
            object_id,
            bytes: Some(bytes),
            max_byte_length: None,
        });
        Ok(BufferId(index))
    }

    pub(super) fn buffer_clone_range_intrinsic(
        &mut self,
        source: BufferId,
        offset: usize,
        length: usize,
    ) -> Result<BufferId> {
        self.work(4)?;
        let available = self.buffer_attached_length(source.0)?;
        if offset.checked_add(length).is_none_or(|end| end > available) {
            return Err(ScriptError::range_error(
                "TypedArray clone range exceeds buffer",
            ));
        }
        let target = self.buffer_allocate_intrinsic(length as u64)?;
        // Both sides of the raw copy are paid after intrinsic allocation and
        // before touching a byte. No callbacks or fallible steps follow debit.
        self.work(1 + byte_work(length).saturating_mul(2))?;
        self.buffer_copy(source.0, target.0, offset, length);
        Ok(target)
    }
}
